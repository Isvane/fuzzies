//! Parallel batch fuzzy search across multiple inputs.
//!
//! Run with: `cargo run --example batch_search`

use fuzzies::{Dictionary, DictionaryError};

fn main() -> Result<(), DictionaryError> {
    let dict = Dictionary::from_iterator([
        "rust",
        "ruby",
        "python",
        "javascript",
        "typescript",
        "golang",
        "elixir",
    ])?;

    let queries = &["rustt", "pythn", "typescrip", "elxir"];

    // Executes queries in parallel via Rayon
    let results = dict
        .batch_search(queries)
        .distance(2)
        .transposition(true)
        .limit(1)
        .execute();

    for (query, match_result) in queries.iter().zip(results) {
        match match_result {
            Ok(matches) if !matches.is_empty() => {
                println!(
                    "'{query}' -> '{}' (distance: {})",
                    matches[0].key, matches[0].distance
                );
            }
            Ok(_) => println!("'{query}' -> No match"),
            Err(e) => eprintln!("Error processing '{query}': {e}"),
        }
    }

    Ok(())
}
