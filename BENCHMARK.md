# Benchmark Data

These were measured using [Criterion.rs](https://github.com/bheisler/criterion.rs) to make sure we're testing the actual search speed, completely separated from memory allocation overhead or test setup.

## TL;DR

1. **Instant rejections:** If a word isn't in the dictionary, fuzzies doesn't waste time checking the rest of the list. It fails fast.
2. **Size doesn't matter:** Exact matches are fast because it only takes time proportional to the word you typed. The dictionary could have 50 million words and exact lookups would still take ~130ns.
3. **Scaling:** If you're building something like a spellchecker that processes whole documents, batching the queries across CPU threads gets the time down to under 20 microseconds per word.

--- 

### Setup
- **Dictionary Size**: 50,000 unique words.
- **Words**: 17-byte strings (e.g., `word_0000a1b2_90b8`).
- **Hardware**: *I5-10300H Laptop CPU*

---

## Building the Dictionary
Before we can search, the engine has to sort the words, remove duplicates, and build the search graph. It processes about 1.8 million words per second.

| Input Size | Time to Build |
| ---: | ---: |
| 1,000 words | `557 µs` |
| 10,000 words | `5.68 ms` |
| 50,000 words | `27.19 ms` |

---

## Exact Lookups (`contains`)
Because of how the data structure works, search time depends on the *length of the word* you're looking for, not how many words are in the dictionary. 

| Query Type | Time | What's happening? |
| :--- | ---: | :--- |
| **Exact Hit** | `130 ns` | Successfully found a 17-character word. |
| **Total Miss** | `6.7 ns` | The engine instantly stops searching the moment it sees a mismatched letter. |

---

## Fuzzy Searching (Handling Typos)
How fast the engine can find close matches. The wider you set the search net, the more paths it has to check.

| Setup | Time | Notes |
| :--- | ---: | :--- |
| 1 Typo allowed | `75.2 µs` | Standard typo correction. |
| 2 Typos allowed | `775.7 µs` | Takes longer because allowing 2 mistakes creates way more possible letter combinations. |
| Search within bounds | `38.6 µs` | Grabbing a specific alphabetical range (like words between "a" and "c"). |
| Allow swapped letters | `77.9 µs` | E.g., typing "teh" instead of "the". Barely affects speed. |
| Prefix Search * | `10.3 ms` | *See note below. |

> **\* Note on the 10ms Prefix Search**: In this specific test, the search prefix was `"word_000"`. Since every single word in our 50,000-word test dataset started with that, the engine had to literally find, calculate distances for, and sort all **50,000 matches**. Processing the entire dictionary in 10ms means it handles an individual match in about 200 nanoseconds. 

---

## Batch Searching (Rayon)
If you need to check a massive list of words at once, the crate uses Rayon to split the work across your CPU cores. (Tested with max 1 typo allowed, returning up to 5 results per query).

| Batch Size | Query Type | Total Time | Average Time Per Query |
| ---: | :--- | ---: | ---: |
| **10** | Exact | `194.3 µs` | ~19.4 µs |
| **100** | Exact | `1.61 ms` | ~16.1 µs |
| **1,000** | Exact | `17.48 ms` | ~17.4 µs |
| **1,000** | 1 Missing Letter | `17.65 ms` | ~17.6 µs |
| **1,000** | 1 Swapped Letter | `5.04 ms` | ~5.0 µs |
| **1,000** | Complete Miss | `6.65 ms` | ~6.6 µs |
