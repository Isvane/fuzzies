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
    map: Set<DictionarySource>,
}

impl Dictionary {
    /// Opens a memory-mapped FST dictionary from an existing file.
    ///
    /// # Examples
    /// ```no_run
    /// # use fuzzies::{Dictionary, DictionaryError};
    /// # fn main() -> Result<(), DictionaryError> {
    /// let dict = Dictionary::open("dict.fst")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DictionaryError> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let map = Set::new(DictionarySource::Mmapped(mmap))?;

        Ok(Self { map })
    }

    /// Creates a dictionary from a static byte slice embedded in the binary.
    ///
    /// Enables single-file executable distribution by baking the FST data
    /// directly into your application using `include_bytes!`.
    ///
    /// # Example
    /// ```ignore
    /// # use fuzzies::Dictionary;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// static DICT_DATA: &[u8] = include_bytes!("../assets/words.fst");
    /// let dict = Dictionary::from_embedded(DICT_DATA)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_embedded(bytes: &'static [u8]) -> Result<Self, DictionaryError> {
        let map = Set::new(DictionarySource::Embedded(bytes))?;
        Ok(Self { map })
    }

    /// Compiles a byte-sorted text file into an immutable binary FST.
    ///
    /// # Example
    /// ```no_run
    /// # use fuzzies::Dictionary;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// Dictionary::build("sorted_words.txt", "dict.fst")?;
    /// # Ok(())
    /// # }
    /// ```
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
    /// Prepares raw source text for processing by [`Self::build`].
    ///
    /// # Example
    /// ```no_run
    /// # use fuzzies::Dictionary;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// Dictionary::sort("unsorted_words.txt")?;
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// # Example
    /// ```no_run
    /// # use fuzzies::Dictionary;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let dict = Dictionary::open("dict.fst")?;
    /// assert!(dict.contains("apple"));
    /// assert!(dict.contains(b"banana")); // Works with byte slices too!
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains(&self, key: impl AsRef<[u8]>) -> bool {
        self.map.contains(key)
    }

    /// Initializes a fuzzy search query builder.
    ///
    /// # Example
    /// ```no_run
    /// # use fuzzies::Dictionary;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let dict = Dictionary::open("dict.fst")?;
    /// let results = dict.search("baxana")
    ///     .distance(2)
    ///     .limit(5)
    ///     .execute()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn search<'a>(&'a self, query: &str) -> SearchBuilder<'a> {
        SearchBuilder::new(self, query)
    }

    /// Executes multiple search queries concurrently.
    ///
    /// # Example
    /// ```no_run
    /// # use fuzzies::Dictionary;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let dict = Dictionary::open("dict.fst")?;
    /// let queries = ["apple", "baxana", "cheriy"];
    /// let batch_results = dict.batch_search(&queries);
    /// # Ok(())
    /// # }
    /// ```
    pub fn batch_search<'a, 'b>(&'a self, queries: &'b [&'b str]) -> BatchSearchBuilder<'a, 'b> {
        BatchSearchBuilder {
            dictionary: self,
            queries,
            limit: 5,
            distance: 1,
            transposition: false,
            prefix: false,
        }
    }
}

/// Query builder for configuring fuzzy searches.
pub struct SearchBuilder<'a> {
    dictionary: &'a Dictionary,
    query: String,
    limit: usize,
    distance: u8,
    transposition: bool,
    prefix: bool,
}

/// Query builder for batch search.
pub struct BatchSearchBuilder<'a, 'b> {
    dictionary: &'a Dictionary,
    queries: &'b [&'b str],
    limit: usize,
    distance: u8,
    transposition: bool,
    prefix: bool,
}

/// Same as [`SearchBuilder`] but for batch searches.
impl<'a, 'b> BatchSearchBuilder<'a, 'b> {
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn distance(mut self, distance: u8) -> Self {
        self.distance = distance;
        self
    }

    pub fn transposition(mut self, transposition: bool) -> Self {
        self.transposition = transposition;
        self
    }

    pub fn prefix(mut self, prefix: bool) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn execute(self) -> Vec<Result<Vec<SearchResult>, DictionaryError>> {
        self.queries
            .par_iter()
            .map(|&query| {
                self.dictionary
                    .search(query)
                    .limit(self.limit)
                    .distance(self.distance)
                    .transposition(self.transposition)
                    .prefix(self.prefix)
                    .execute()
            })
            .collect()
    }
}

impl<'a> SearchBuilder<'a> {
    /// Defaults: `limit = 5`, `distance = 1`, `transposition = false`.
    pub fn new(dictionary: &'a Dictionary, query: impl Into<String>) -> Self {
        Self {
            dictionary,
            query: query.into(),
            limit: 5,
            distance: 1,
            transposition: false,
            prefix: false,
        }
    }

    /// Max number of results to return.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Maximum Levenshtein distance for fuzzy searching (hard-capped at 2).
    pub fn distance(mut self, distance: u8) -> Self {
        self.distance = distance.min(2);
        self
    }

    /// Whether to allow adjacent character swaps (e.g., "teh" -> "the").
    pub fn transposition(mut self, transposition: bool) -> Self {
        self.transposition = transposition;
        self
    }

    /// Whether to perform a prefix fuzzy search.
    pub fn prefix(mut self, prefix: bool) -> Self {
        self.prefix = prefix;
        self
    }

    /// Evaluates the fuzzy search against the dictionary.
    pub fn execute(self) -> Result<Vec<SearchResult>, DictionaryError> {
        let builder = LevenshteinAutomatonBuilder::new(self.distance, self.transposition);

        let dfa = FstDfaWrapper(if self.prefix {
            builder.build_prefix_dfa(&self.query)
        } else {
            builder.build_dfa(&self.query)
        });

        let mut query_counts = [0i16; 256];
        for &b in self.query.as_bytes() {
            query_counts[b as usize] += 1;
        }

        let mut heap = BinaryHeap::with_capacity(self.limit);
        let mut stream = self.dictionary.map.search(&dfa).into_stream();

        while let Some(key_bytes) = stream.next() {
            let mut state = dfa.start();
            for &byte in key_bytes {
                state = dfa.accept(&state, byte);
            }

            let dist = match dfa.0.distance(state) {
                Distance::Exact(d) => d,
                _ => self.distance,
            };

            let mut char_diff = 0u16;
            let mut counts = query_counts;
            for &b in key_bytes {
                counts[b as usize] -= 1;
            }
            for c in counts {
                char_diff += c.abs() as u16;
            }

            let candidate = (dist, char_diff, key_bytes);

            if heap.len() < self.limit {
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
                    is_exact: dist == 0,
                    key: String::from_utf8(bytes)?,
                    distance: dist,
                })
            })
            .collect::<Result<_, DictionaryError>>()?;

        Ok(results)
    }
}

/// A matched item from a fuzzy search.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SearchResult {
    /// True if Levenshtein distance is 0.
    pub is_exact: bool,
    /// The matched string.
    pub key: String,
    /// Levenshtein distance to the query.
    pub distance: u8,
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
}

impl AsRef<[u8]> for DictionarySource {
    fn as_ref(&self) -> &[u8] {
        match self {
            DictionarySource::Mmapped(mmap) => mmap,
            DictionarySource::Embedded(slice) => slice,
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
