// An interactive spellchecker demonstrating real-time fuzzy matching.
// How to run:
// cargo run --release --example repl

use fuzzies::{Dictionary, DictionaryError};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), DictionaryError> {
    let fst_path = "tech_dictionary.fst";

    setup_dictionary(fst_path)?;

    println!("Loading dictionary into memory...");
    let dict = Dictionary::open(fst_path)?;

    println!("=======================================================");
    println!(" Fuzzies Interactive Search REPL");
    println!("=======================================================");
    println!("Type a word to find fuzzy matches. (Type 'quit' to stop)\n");

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
            .distance(2) // Catch moderate typos
            .transposition(true) // Catch swapped letters (e.g., 'teh' -> 'the')
            .limit(5) // Only show the top 5 suggestions
            .execute()?;

        let duration = start_time.elapsed();

        // Display results
        if results.is_empty() {
            println!("No matches found for '{}'.", query);
        } else {
            println!("Found {} matches in {:?}", results.len(), duration);
            for (i, result) in results.into_iter().enumerate() {
                // Highlight exact matches visually
                let exact_marker = if result.is_exact { " ✨ [EXACT]" } else { "" };
                println!("    {}. {}{}", i + 1, result.key, exact_marker);
            }
        }
        println!();
    }

    Ok(())
}

/// Helper function to generate a practical dataset if one doesn't exist.
fn setup_dictionary(fst_path: &str) -> Result<(), DictionaryError> {
    if Path::new(fst_path).exists() {
        return Ok(());
    }

    println!("First run detected. Generating a sample tech dictionary...");
    let txt_path = "tech_dictionary.txt";
    let mut file = File::create(txt_path)?;

    // A curated list of common programming/tech words that people often misspell
    let words = [
        "algorithm",
        "application",
        "authentication",
        "authorization",
        "bandwidth",
        "binary",
        "boolean",
        "browser",
        "cache",
        "compiler",
        "connection",
        "cryptography",
        "database",
        "debugger",
        "deployment",
        "dictionary",
        "encryption",
        "endpoint",
        "environment",
        "execution",
        "framework",
        "function",
        "gateway",
        "gigabyte",
        "hardware",
        "heuristic",
        "hostname",
        "hypertext",
        "iteration",
        "interface",
        "javascript",
        "json",
        "kernel",
        "keyboard",
        "latency",
        "linux",
        "macintosh",
        "macro",
        "memory",
        "microservice",
        "network",
        "node",
        "object",
        "operating",
        "packet",
        "parameter",
        "password",
        "performance",
        "query",
        "queue",
        "recursion",
        "repository",
        "router",
        "server",
        "software",
        "syntax",
        "system",
        "terminal",
        "thread",
        "token",
        "topology",
        "transaction",
        "ubuntu",
        "unicode",
        "variable",
    ];

    for word in words {
        writeln!(file, "{}", word)?;
    }

    // Prepare the raw data and compile the FST
    Dictionary::sort(txt_path)?;
    Dictionary::build(txt_path, fst_path)?;
    println!("Dictionary built successfully!\n");

    Ok(())
}
