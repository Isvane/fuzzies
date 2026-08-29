//! In-memory dictionary build directly from a string slice.
//!
//! Run with: `cargo run --example in_memory`

use fuzzies::{Dictionary, DictionaryError};

fn main() -> Result<(), DictionaryError> {
    // `from_iterator` automatically sorts & deduplicates
    let words = vec!["apple", "apricot", "application", "banana", "band"];
    let dict = Dictionary::from_iterator(words)?;

    let query = "applicaion"; // Typo: missing 't'
    if let Some(s) = dict.suggest(query)? {
        println!(
            "Did you mean: '{}'? (similarity: {:.0}%)",
            s.key,
            s.similarity(query) * 100.0
        );
    }

    Ok(())
}
