# Fuzzies

**Fuzzies** is a fast fuzzy string matching engine built on `fst` and `levenshtein-automata`. It comes pre-configured with memory-mapped storage (`memmap2`), parallel batch search (`rayon`), and transposition support.

[![Crates.io](https://img.shields.io/crates/v/fuzzies.svg)](https://crates.io/crates/fuzzies)
[![Docs.rs](https://docs.rs/fuzzies/badge.svg)](https://docs.rs/fuzzies)
[![Crates.io](https://img.shields.io/crates/l/fuzzies)](https://github.com/Isvane/fuzzies/blob/main/LICENSE)

More information about this crate can be found in the [crate documentation](https://docs.rs/fuzzies)

---

## Features

* **Zero-Copy Memory Mapping**: Instantly query huge disk-backed datasets via `memmap2`.
* **Out-of-the-Box Transpositions**: Handles Damerau-Levenshtein transposition edits natively alongside standard insertion, deletion, and substitution.
* **Parallel Batching**: Built-in multi-threaded query execution powered by `rayon`.
* **Zero Boilerplate**: Avoid manual glue code between `fst` state machines and `levenshtein-automata`.

---

## Installation

```bash
cargo add fuzzies
```

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

    // Perform a fuzzy search with a max typo distance of 2 and limit of 5 results
    let results = dict.search("banaan")
        .distance(2)
        .transposition(true) // Handles adjacent swaps (e.g., "teh" -> "the")
        .prefix(false)       // Set to true for prefix fuzzy lookups
        // .ge("a").lt("e")  // Optionally restrict search bounds (e.g., 'a' <= key < 'e')
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

### Embedding Data

If you don't want to manage external `.fst` files on disk, embed the dataset directly into your application:

```rust, ignore
static DICT_DATA: &[u8] = include_bytes!("../assets/words.fst");
let dict = Dictionary::from_embedded(DICT_DATA)?;
```

---

## 🎈 Performance

The following benchmarks were gathered using Criterion on an **Intel Core i5-10300H** (4 cores / 8 threads). You can re-run these on your hardware with `cargo bench`.

> [!NOTE] 
> Running `cargo bench` on the published crate executes against a small, dynamically generated dataset. The 106,000-word benchmarks shown below were gathered independently using a local dictionary.

### Setup & Initialization

| Operation | 1,000 Entries | 106,000 Entries |
|-----------|------:|------:|
| `Dictionary::sort` | 36.87 µs | — |
| `Dictionary::build` | 130.97 µs | — |
| `Dictionary::open` | 2.78 µs | 6.96 µs |
| `Dictionary::from_embedded`| 11.75 ns | 11.77 ns |
| `Dictionary::len` / `is_empty` | ~0.45 ns | ~0.45 ns |

### Search & Queries

| Operation | 1,000 Entries | 106,000 Entries | Scaling Factor |
|-----------|---------------------:|----------------------:|---------------:|
| contains (Hit) | 34.50 ns | 63.94 ns | ~1.9x |
| contains (Miss) | 11.13 ns | 97.84 ns | ~8.8x |
| Exact Search (dist = 0) | 2.01 µs | 4.26 µs | ~2.1x |
| Fuzzy Search (dist = 1) | 8.31 µs | 64.55 µs | ~7.8x |
| Prefix Search | 5.43 µs | 125.95 µs | Result-size bound |
| Range Search (`'b'..='c'`) | 4.19 µs | 637.90 µs | Result-size bound |
| Batch (1,000 queries) | 4.08 ms (4.0 µs/q) | 14.88 ms (14.8 µs/q) | ~3.6x |

---

## Safety

This crate uses `unsafe` in a single location:

* **Memory-Mapped I/O (`Dictionary::open`)**: Calls `memmap2::Mmap::map(&file)` to map FST data directly from disk into memory. 

### Invariants & Requirements

While memory mapping allows zero-copy lookups with near-instant load times, **undefined behavior can occur** if the underlying file on disk is modified, truncated, or corrupted by another process while the `Dictionary` is active in memory.

If your application operates in an environment where external processes might mutate file assets concurrently, consider using `Dictionary::from_embedded` or `Dictionary::from_iterator` instead to ensure memory safety guarantees.

---

## License

This project is licensed under the [MIT license.](LICENSE)
