//! Interactive REPL demonstrating fuzzy search.
//!
//! Automatically builds an FST dictionary on first run
//! and lets you query terms with up to 2 edits/transpositions in real-time.
//!
//! Run with: `cargo run --example repl --release`

use fuzzies::{Dictionary, DictionaryError};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), DictionaryError> {
    let fst_path = "dict.fst";

    setup_dictionary(fst_path)?;

    println!("Loading dictionary into memory...");
    let dict = Dictionary::open(fst_path)?;
    println!("Dictionary loaded successfully!");

    println!("\nType a word to find fuzzy matches. (Type 'quit' to stop)\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("search > ");
        stdout.flush()?;

        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        let query = input.trim();

        if query.eq_ignore_ascii_case("quit") || query.eq_ignore_ascii_case("exit") {
            println!("Goodbye!");
            break;
        }
        if query.is_empty() {
            continue;
        }

        let start_time = Instant::now();

        let results = dict
            .search(query)
            .distance(2)
            .transposition(true)
            .limit(5)
            .execute()?;

        let duration = start_time.elapsed();

        if results.is_empty() {
            println!("No matches found for '{query}'.");
        } else {
            println!("Found {} matches in {duration:?}", results.len());
            for (i, result) in results.into_iter().enumerate() {
                let exact_marker = if result.is_exact() { " [EXACT]" } else { "" };
                println!("    {}. {}{}", i + 1, result.key, exact_marker);
            }
        }
        println!();
    }

    Ok(())
}

fn setup_dictionary(fst_path: &str) -> Result<(), DictionaryError> {
    if Path::new(fst_path).exists() {
        return Ok(());
    }

    println!("First run detected. Generating a sample dictionary...");
    let txt_path = "dict.txt";
    let mut file = File::create(txt_path)?;

    let words = [
        "apple",
        "banana",
        "mango",
        "rust",
        "computer",
        "book",
        "bar",
        "bear",
        "bool",
        "foo",
        "bar",
        "love",
        "programming",
        "programmer",
        "profanity",
    ];

    for word in words {
        writeln!(file, "{word}")?;
    }

    Dictionary::sort(txt_path)?;
    Dictionary::build(txt_path, fst_path)?;
    println!("Dictionary built successfully!\n");

    Ok(())
}
