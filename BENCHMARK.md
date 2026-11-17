Measured using [Criterion.rs](https://github.com/bheisler/criterion.rs).

## TL;DR

1. **Instant loading:** Opening a pre-built `.fst` dictionary takes `10.55 µs`. Building one from 50,000 raw words takes `18.94 ms` (about 2.64 million words per second).
2. **Fast exact lookups:** Checking if a word exists takes `160 ns` for a hit and `140 ns` for a miss. Lookup speed depends on word length, not dictionary size.
3. **Predictable fuzzy search:** Finding words within 1 typo takes about `100 µs` to `105 µs`. Adding alphabetical bounds drops that time to `27.5 µs`.
4. **Multi-core batch scaling:** Running batch searches across all CPU cores drops per-query time to roughly `18 µs` to `23 µs`, reaching up to `54.9K` queries per second.

---

### Setup

- **Dataset**: Real English words from `assets/words.fst` (50,000 words used for construction and query sampling).
- **Hardware**: Intel i5-10300H laptop CPU.
- **Environment**: Single-thread benchmarks pinned to `CPU 3` (ASLR disabled, real-time priority). Batch benchmarks run across all available CPU cores.

---

## Loading and Building the Dictionary

You can load a compiled FST file directly from disk, or build a dictionary in memory from an iterator of strings. Building from raw strings sorts the words, removes duplicates, and constructs the search graph.

| Method | Input | Time | Throughput |
| :--- | :--- | ---: | ---: |
| `Dictionary::open` | Pre-built `.fst` file | `10.55 µs` | N/A (zero-copy load) |
| `Dictionary::from_iterator` | 50,000 words | `18.94 ms` | 2.64M words/s |

---

## Exact Lookups (`contains`)

Exact lookup time depends on the length of the query word, not how many words are in the dictionary.

| Query Type | Time | Throughput | Notes |
| :--- | ---: | ---: | :--- |
| **Exact Hit** | `160.4 ns` | 6.24M ops/s | Matches a known English word like `"algorithm"`. |
| **Near Miss** | `140.4 ns` | 7.12M ops/s | Keeps a real word prefix and changes the ending (e.g., `"qx"`), stopping once the path breaks. |

---

## Fuzzy Searching (Single Query)

Fuzzy search finds dictionary words within a set edit distance. Wider tolerances and prefix matching explore more paths in the graph, which increases search time.

| Test Case | Time | Notes |
| :--- | ---: | :--- |
| **1 typo allowed** | `104.5 µs` | Default single-edit fuzzy search. |
| **2 typos allowed** | `819.6 µs` | Allowing 2 edits branches into many more candidate paths. |
| **Swapped letters off** | `100.2 µs` | 1 typo allowed, adjacent letter swaps disabled. |
| **Swapped letters on** | `102.9 µs` | Allows adjacent swaps (e.g., `"teh"` for `"the"`) with almost no speed penalty. |
| **Prefix search off** | `96.6 µs` | Searches using the first half of a target word as a full word (1 typo). |
| **Prefix search on** | `178.9 µs` | Matches all words starting with that half-word prefix (1 typo). |
| **Result limit (1, 5, or 50)** | `99.3 to 99.9 µs` | Limiting returned results has almost no effect on search speed. |
| **Alphabetical bounds** | `27.5 µs` | Restricting search between lower (`ge`) and upper (`le`) bounds cuts search time by ~73%. |

---

## Batch Searching (Rayon)

`batch_search` splits query slices across all available CPU cores using Rayon. All tests below allow up to 1 typo and return up to 5 results per query.

| Query Type | Batch Size | Total Time | Avg Per Query | Throughput |
| :--- | ---: | ---: | ---: | ---: |
| **Exact match** | 10 | `188.3 µs` | `18.8 µs` | 53.10K queries/s |
| **Exact match** | 100 | `2.13 ms` | `21.3 µs` | 47.02K queries/s |
| **Exact match** | 1,000 | `22.65 ms` | `22.7 µs` | 44.15K queries/s |
| **1 missing letter** | 10 | `182.1 µs` | `18.2 µs` | 54.92K queries/s |
| **1 missing letter** | 100 | `2.08 ms` | `20.8 µs` | 48.18K queries/s |
| **1 missing letter** | 1,000 | `22.17 ms` | `22.2 µs` | 45.11K queries/s |
| **1 replaced letter** | 10 | `186.4 µs` | `18.6 µs` | 53.66K queries/s |
| **1 replaced letter** | 100 | `2.08 ms` | `20.8 µs` | 48.02K queries/s |
| **1 replaced letter** | 1,000 | `22.39 ms` | `22.4 µs` | 44.67K queries/s |
| **2 swapped letters** | 10 | `196.4 µs` | `19.6 µs` | 50.91K queries/s |
| **2 swapped letters** | 100 | `2.11 ms` | `21.1 µs` | 47.33K queries/s |
| **2 swapped letters** | 1,000 | `23.21 ms` | `23.2 µs` | 43.08K queries/s |
| **Complete miss** | 10 | `185.5 µs` | `18.6 µs` | 53.91K queries/s |
| **Complete miss** | 100 | `2.09 ms` | `20.9 µs` | 47.95K queries/s |
| **Complete miss** | 1,000 | `22.34 ms` | `22.3 µs` | 44.77K queries/s |
