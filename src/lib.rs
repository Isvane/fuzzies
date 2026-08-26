#![doc = include_str!("../README.md")]

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use fst::{Automaton, IntoStreamer, Set, SetBuilder, Streamer};
use levenshtein_automata::{DFA, Distance, LevenshteinAutomatonBuilder};
use memmap2::Mmap;
use rayon::prelude::*;

mod test;

#[derive(thiserror::Error, Debug)]
pub enum DictionaryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("FST error: {0}")]
    Fst(#[from] fst::Error),

    #[error("Invalid UTF-8 sequence: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Memory-mapped FST dictionary for fuzzy string lookups.
pub struct Dictionary {
    source: DictionarySource,
}

impl Dictionary {
    /// Opens a memory-mapped FST dictionary from an existing file.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DictionaryError> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        Set::new(&*mmap)?;

        Ok(Self {
            source: DictionarySource::Mmapped(mmap),
        })
    }

    /// Creates a dictionary from a static byte slice embedded in the binary.
    pub fn from_embedded(bytes: &'static [u8]) -> Result<Self, DictionaryError> {
        Set::new(bytes)?;
        Ok(Self {
            source: DictionarySource::Embedded(bytes),
        })
    }

    /// Creates a dictionary in memory from an iterator of keys.
    pub fn from_iterator<T, I>(iter: I) -> Result<Self, DictionaryError>
    where
        T: AsRef<[u8]>,
        I: IntoIterator<Item = T>,
    {
        let mut keys: Vec<Vec<u8>> = iter
            .into_iter()
            .map(|item| item.as_ref().to_vec())
            .collect();

        keys.sort_unstable();
        keys.dedup();

        let mut builder = SetBuilder::memory();
        builder.extend_iter(keys)?;

        let bytes = builder.into_inner()?;

        Set::new(bytes.as_slice())?;

        Ok(Self {
            source: DictionarySource::Owned(bytes),
        })
    }

    /// Compiles a byte-sorted text file into an immutable binary FST.
    pub fn build(
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
    ) -> Result<(), DictionaryError> {
        let mut reader = BufReader::new(File::open(input_path)?);
        let mut build = SetBuilder::new(BufWriter::new(File::create(output_path)?))?;
        let mut line = String::new();

        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                build.insert(trimmed)?;
            }
            line.clear();
        }

        build.finish()?;
        Ok(())
    }

    /// Sorts a newline-delimited text file in-place by byte order.
    pub fn sort(path: impl AsRef<Path>) -> Result<(), DictionaryError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        let mut lines: Vec<&str> = content.lines().collect();
        lines.sort_unstable();

        let mut writer = BufWriter::new(File::create(path)?);
        for line in lines {
            if !line.is_empty() {
                writeln!(writer, "{}", line)?;
            }
        }
        Ok(())
    }

    /// Returns `true` if the dictionary contains the exact key.
    pub fn contains(&self, key: impl AsRef<[u8]>) -> bool {
        self.as_set().contains(key)
    }

    /// Returns the dictionary length.
    pub fn len(&self) -> usize {
        self.as_set().len()
    }

    /// Returns `true` if the dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.as_set().is_empty()
    }

    /// Initializes a fuzzy search query builder.
    pub fn search<'a>(&'a self, query: &str) -> SearchBuilder<'a, String> {
        SearchBuilder::new(self, query)
    }

    /// Executes multiple search queries concurrently.
    pub fn batch_search<'a, 'b>(&'a self, queries: &'b [&'b str]) -> BatchSearchBuilder<'a, 'b> {
        SearchBuilder {
            dictionary: self,
            query: queries,
            options: SearchOptions::default(),
        }
    }

    /// Search implementation shared by single and batch queries.
    fn executes(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>, DictionaryError> {
        let builder = LevenshteinAutomatonBuilder::new(options.distance, options.transposition);

        let dfa = FstDfaWrapper(if options.prefix {
            builder.build_prefix_dfa(query)
        } else {
            builder.build_dfa(query)
        });

        let mut query_counts = [0i16; 256];
        for &b in query.as_bytes() {
            query_counts[b as usize] += 1;
        }

        let mut heap: BinaryHeap<(u8, u16, Vec<u8>)> = BinaryHeap::with_capacity(options.limit);
        let set = self.as_set();

        let mut fst_search = set.search(&dfa);

        if let Some(bound) = &options.ge {
            fst_search = fst_search.ge(bound);
        }
        if let Some(bound) = &options.gt {
            fst_search = fst_search.gt(bound);
        }
        if let Some(bound) = &options.le {
            fst_search = fst_search.le(bound);
        }
        if let Some(bound) = &options.lt {
            fst_search = fst_search.lt(bound);
        }

        let mut stream = fst_search.into_stream();

        while let Some(key_bytes) = stream.next() {
            let mut state = dfa.start();
            for &byte in key_bytes {
                state = dfa.accept(&state, byte);
            }

            let dist = match dfa.0.distance(state) {
                Distance::Exact(d) => d,
                _ => options.distance,
            };

            if heap.len() == options.limit
                && let Some(worst) = heap.peek()
                && dist > worst.0
            {
                continue;
            }

            let mut char_diff = 0u16;
            let mut counts = query_counts;
            for &b in key_bytes {
                counts[b as usize] -= 1;
            }
            for c in counts {
                char_diff += c.unsigned_abs();
            }

            let candidate = (dist, char_diff, key_bytes);

            if heap.len() < options.limit {
                heap.push((dist, char_diff, key_bytes.to_vec()));
            } else if let Some(mut worst) = heap.peek_mut()
                && candidate < (worst.0, worst.1, worst.2.as_slice())
            {
                *worst = (dist, char_diff, key_bytes.to_vec());
            }
        }

        let sorted_elements = heap.into_sorted_vec();

        let results: Vec<_> = sorted_elements
            .into_iter()
            .map(|(dist, _char_diff, bytes)| {
                Ok(SearchResult {
                    key: String::from_utf8(bytes)?,
                    distance: dist,
                })
            })
            .collect::<Result<_, DictionaryError>>()?;

        Ok(results)
    }

    /// Helper to get a zero-cost FST view.
    /// Unwrapping is safe because we validate the FST at construction time.
    #[inline]
    fn as_set(&self) -> Set<&[u8]> {
        Set::new(self.source.as_ref()).expect("FST data is corrupted")
    }
}

/// Internal configuration options for search queries.
#[derive(Clone, Debug)]
struct SearchOptions {
    limit: usize,
    distance: u8,
    transposition: bool,
    prefix: bool,
    ge: Option<Vec<u8>>,
    gt: Option<Vec<u8>>,
    le: Option<Vec<u8>>,
    lt: Option<Vec<u8>>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 5,
            distance: 1,
            transposition: false,
            prefix: false,
            ge: None,
            gt: None,
            le: None,
            lt: None,
        }
    }
}

/// Query builder for configuring single and batch fuzzy searches.
pub struct SearchBuilder<'a, Q = String> {
    dictionary: &'a Dictionary,
    query: Q,
    options: SearchOptions,
}

/// Query builder type alias for batch searches.
pub type BatchSearchBuilder<'a, 'b> = SearchBuilder<'a, &'b [&'b str]>;

impl<'a, Q> SearchBuilder<'a, Q> {
    /// Max number of results to return.
    pub fn limit(mut self, limit: usize) -> Self {
        self.options.limit = limit;
        self
    }

    /// Maximum Levenshtein distance for fuzzy searching (hard-capped at 2).
    pub fn distance(mut self, distance: u8) -> Self {
        self.options.distance = distance.min(2);
        self
    }

    /// Whether to allow adjacent character swaps (e.g., "teh" -> "the").
    pub fn transposition(mut self, transposition: bool) -> Self {
        self.options.transposition = transposition;
        self
    }

    /// Whether to perform a prefix fuzzy search.
    pub fn prefix(mut self, prefix: bool) -> Self {
        self.options.prefix = prefix;
        self
    }

    pub fn ge(mut self, bound: impl AsRef<[u8]>) -> Self {
        self.options.ge = Some(bound.as_ref().to_vec());
        self
    }

    pub fn gt(mut self, bound: impl AsRef<[u8]>) -> Self {
        self.options.gt = Some(bound.as_ref().to_vec());
        self
    }

    pub fn le(mut self, bound: impl AsRef<[u8]>) -> Self {
        self.options.le = Some(bound.as_ref().to_vec());
        self
    }

    pub fn lt(mut self, bound: impl AsRef<[u8]>) -> Self {
        self.options.lt = Some(bound.as_ref().to_vec());
        self
    }
}

impl<'a> SearchBuilder<'a, String> {
    /// Defaults: `limit = 5`, `distance = 1`, `transposition = false`.
    pub fn new(dictionary: &'a Dictionary, query: impl Into<String>) -> Self {
        Self {
            dictionary,
            query: query.into(),
            options: SearchOptions::default(),
        }
    }

    /// Evaluates the fuzzy search against the dictionary.
    pub fn execute(self) -> Result<Vec<SearchResult>, DictionaryError> {
        self.dictionary.executes(&self.query, &self.options)
    }
}

impl<'a, 'b> SearchBuilder<'a, &'b [&'b str]> {
    /// Evaluates multiple search queries concurrently.
    pub fn execute(self) -> Vec<Result<Vec<SearchResult>, DictionaryError>> {
        self.query
            .par_iter()
            .map(|&q| self.dictionary.executes(q, &self.options))
            .collect()
    }
}

/// A matched item from a fuzzy search.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SearchResult {
    /// The matched string.
    pub key: String,
    /// Levenshtein distance to the query.
    pub distance: u8,
}

impl SearchResult {
    #[inline]
    pub fn is_exact(&self) -> bool {
        self.distance == 0
    }
}

impl Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (distance: {})", self.key, self.distance)
    }
}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance
            .cmp(&other.distance)
            .then_with(|| self.key.cmp(&other.key))
    }
}

/// Underlying storage strategy for the dictionary data.
enum DictionarySource {
    Mmapped(Mmap),
    Embedded(&'static [u8]),
    Owned(Vec<u8>),
}

impl AsRef<[u8]> for DictionarySource {
    fn as_ref(&self) -> &[u8] {
        match self {
            DictionarySource::Mmapped(mmap) => mmap,
            DictionarySource::Embedded(slice) => slice,
            DictionarySource::Owned(vec) => vec,
        }
    }
}

/// Adapts a Levenshtein [`DFA`] to the [`fst::Automaton`] trait ecosystem.
struct FstDfaWrapper(DFA);

impl fst::Automaton for FstDfaWrapper {
    type State = u32;

    #[inline]
    fn start(&self) -> Self::State {
        self.0.initial_state()
    }

    #[inline]
    fn is_match(&self, state: &Self::State) -> bool {
        matches!(self.0.distance(*state), Distance::Exact(_))
    }

    #[inline]
    fn accept(&self, state: &Self::State, byte: u8) -> Self::State {
        self.0.transition(*state, byte)
    }

    #[inline]
    fn can_match(&self, state: &Self::State) -> bool {
        *state != levenshtein_automata::SINK_STATE
    }
}
