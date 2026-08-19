# Fuzzies

**Fuzzies** is a fast, friendly integration layer that bridges the gap between low-level finite state transducers (`fst`) and Levenshtein automata, saving you from writing tedious boilerplate.

[![Crates.io](https://img.shields.io/crates/v/fuzzies.svg)](https://crates.io/crates/fuzzies)
[![Docs.rs](https://docs.rs/fuzzies/badge.svg)](https://docs.rs/fuzzies)
[![Crates.io](https://img.shields.io/crates/l/fuzzies)](https://github.com/Isvane/fuzzies/blob/main/LICENSE)

More information about this crate can be found in the [crate documentation](https://docs.rs/fuzzies)

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
> Running cargo bench on the published crate executes against a small, dynamically generated dataset. The 106,000-word benchmarks shown below were gathered independently using a local dictionary.

| Operation | 1,000 Entries | 106,000 Entries | Scaling Factor |
|-----------|---------------------:|----------------------:|---------------:|
| contains (Hit) | 34.60 ns | 65.00 ns | ~1.8x |
| contains (Miss) | 11.08 ns | 100.06 ns | ~9.0x |
| Exact Search (dist = 0) | 2.18 µs | 4.48 µs | ~2.0x |
| Fuzzy Search (dist = 1) | 7.97 µs | 61.58 µs | ~7.7x |
| Prefix Search | 5.53 µs | 125.36 µs | Result-size bound |
| Range Search (`'b'..='c'`) | 4.35 µs | 626.37 µs | Result-size bound |
| Batch (1,000 queries) | 4.02 ms (4.0 µs/q) | 14.88 ms (14.8 µs/q) | ~3.7x |

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
