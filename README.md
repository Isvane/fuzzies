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

## Built with Fuzzies

Check out real-world projects utilizing `fuzzies`:

* **[Mamoru](https://github.com/Isvane/mamoru)**: A blazing-fast Git `commit-msg` hook that embeds a compiled dictionary of over 106,000 words to instantly catch and block typos before they make it into your version control history.

---

## 🎈 Performance

The following benchmarks were gathered using Criterion on an **Intel Core i5-10300H** (4 cores / 8 threads). You can re-run these on your hardware with `cargo bench`.

### Metadata & Operations

```ignore
Dictionary Operations/len                448.12 ps/iter (+/- 2.10 ps)
Dictionary Operations/contains (Hit)      34.60 ns/iter (+/- 0.15 ns)
Dictionary Operations/contains (Miss)     11.08 ns/iter (+/- 0.08 ns)
```

### Loading & Creation

```ignore
Dictionary Setup/from_embedded            11.14 ns/iter (+/- 0.05 ns)
Dictionary Setup/open (Mmap)              3.68 µs/iter (+/- 0.02 µs)
Dictionary Setup/sort (In-place)        109.05 µs/iter (+/- 0.81 µs)
Dictionary Setup/build                  192.88 µs/iter (+/- 1.12 µs)
```

### Search Performance

```ignore
Dictionary Search/Exact (dist = 0)        2.18 µs/iter (+/- 0.01 µs)
Dictionary Search/Range Bounded           4.35 µs/iter (+/- 0.02 µs)
Dictionary Search/Prefix                  5.53 µs/iter (+/- 0.03 µs)
Dictionary Search/Fuzzy (dist = 1)        7.97 µs/iter (+/- 0.04 µs)
```

### Parallel Batch Search (Rayon)

```ignore
Rayon Parallel Batch/100 queries        429.00 µs/iter (+/- 1.82 µs)  [~4.29 µs/query]
Rayon Parallel Batch/500 queries          2.00 ms/iter (+/- 0.01 ms)  [~4.01 µs/query]
Rayon Parallel Batch/1000 queries         4.02 ms/iter (+/- 0.02 ms)  [~4.02 µs/query]
```

---

## License

This project is licensed under the [MIT license.](LICENSE)
