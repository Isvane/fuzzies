# fuzzies

[![Crates.io](https://img.shields.io/crates/v/fuzzies.svg)](https://crates.io/crates/fuzzies)
[![NPM Version](https://img.shields.io/npm/v/fuzzies-node.svg)](https://www.npmjs.com/package/fuzzies-node)
[![Docs.rs](https://docs.rs/fuzzies/badge.svg)](https://docs.rs/fuzzies)
[![Crates.io](https://img.shields.io/crates/l/fuzzies)](https://github.com/Isvane/fuzzies/blob/main/LICENSE)

More information about this crate can be found in the [crate documentation](https://docs.rs/fuzzies). 

---

## Installation

```bash
cargo add fuzzies
```

_See the [Node.js README](bindings/node/README.md) for Node-specific APIs and its usage details._

---

## Example

```rust, no_run
use fuzzies::{Dictionary, DictionaryError};

fn main() -> Result<(), DictionaryError> {
    // Prepare your raw text file (must be sorted lexicographically)
    // Fuzzies provides a handy in-place sorter for convenience:
    Dictionary::sort("words.txt")?;

    // Build the immutable binary FST from the sorted text file
    Dictionary::build("words.txt", "words.fst")?;

    // Load the dictionary (memory-mapped from disk)
    let dict = Dictionary::open("words.fst")?;

    // Check for exact matches instantly
    if dict.contains("banana") {
        println!("Exact match found!");
    }

    // Quick single-suggestion lookup
    if let Some(top_match) = dict.suggest("banaan")? {
        println!("Best suggestion: {}", top_match.key);
    }

    // Perform a fuzzy search with custom options
    let results = dict.search("banaan")
        .distance(2)
        .transposition(true) // Handles adjacent swaps (e.g., "teh" -> "the")
        .prefix(false)       // Set to true for prefix fuzzy lookups
        .ge("a")             // Optionally restrict search bounds (key >= "a")
        .lt("e")             // (key < "e")
        .limit(5)
        .execute()?;
    
    for result in results {
        println!("Found: {}", result);
    }

    // Batch search (multithreaded, defaults to a distance of 1)
    let queries = vec!["aple", "baxana", "cherri"];
    let batch_results = dict.batch_search(&queries).execute();

    for (query, result) in queries.iter().zip(batch_results) {
        match result {
            Ok(matches) => println!("Query '{}' found {} matches", query, matches.len()),
            Err(e) => eprintln!("Error searching for '{}': {}", query, e),
        }
    }

    Ok(())
}
```

If you don't want to manage external `.fst` files on disk, you can construct a `Dictionary` directly:

```rust, ignore
// From an embedded FST binary slice:
static DICT_DATA: &[u8] = include_bytes!("../assets/words.fst");
let dict = Dictionary::from_embedded(DICT_DATA)?;

// From an iterator of strings (automatically sorted and deduplicated):
let words = vec!["apple", "banana", "cherry"];
let dict = Dictionary::from_iterator(words)?;
```

_See [examples](examples/) for more._

---

## 🎈 Performance

Single-thread benchmarks pinned to `CPU 3` (ASLR disabled, real-time priority). Batch benchmarks run across all available CPU cores.

| Operation | Time | Throughput |
| --- | --- | --- |
| `Dictionary::open (.fst)` | 10.55 µs | - |
| `Dictionary::from_iterator (50,000 words)` | 18.94 ms | 2.64M words/s |
| `Dictionary::contains (exact hit)` | 160.4 ns | 6.24M ops/s |
| `Dictionary::search (1 typo)` | 104.5 µs | - |
| `Dictionary::batch_search (1,000 queries, 1 typo)` | 22.65 ms | 44.15K queries/s |

See [Benchmarks](BENCHMARK.md) for more.

---

## Safety

This crate uses `unsafe` in a single location:

* **Memory-Mapped I/O (`Dictionary::open`)**: Calls `memmap2::Mmap::map(&file)` to map FST data directly from disk into memory. 

---

## License

This project is licensed under the [MIT license.](LICENSE)
