//! Dictionary Features Demo
//!
//! Run with:
//! ```sh
//! cargo run --example demo
//! ```

use fuzzies::{Dictionary, SearchResult};
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Fuzzies Dictionary Feature Demo ===\n");

    let temp_dir = std::env::temp_dir();
    let raw_file_path = temp_dir.join("demo_words_unsorted.txt");
    let fst_file_path = temp_dir.join("demo_words.fst");

    println!("[1] Preparing and sorting input text file...");
    let unsorted_words = vec![
        "cherry",
        "banana",
        "apple",
        "apricot",
        "blackberry",
        "blueberry",
        "peach",
        "pear",
        "plum",
        "pineapple",
    ];

    {
        let mut file = File::create(&raw_file_path)?;
        for word in &unsorted_words {
            writeln!(file, "{}", word)?;
        }
    }

    Dictionary::sort(&raw_file_path)?;
    println!("    ✓ Text file sorted at {:?}", raw_file_path);

    println!("\n[2] Compiling binary FST file...");
    Dictionary::build(&raw_file_path, &fst_file_path)?;
    println!("    ✓ FST compiled at {:?}", fst_file_path);

    println!("\n[3] Opening dictionary via Mmap...");
    let dict = Dictionary::open(&fst_file_path)?;

    println!("\n[4] Dictionary metadata & exact lookup checks:");
    println!("    - Total items (len): {}", dict.len());
    println!("    - Is empty: {}", dict.is_empty());
    println!("    - Contains string 'apple': {}", dict.contains("apple"));
    println!(
        "    - Contains byte slice b\"banana\": {}",
        dict.contains(b"banana")
    );
    println!(
        "    - Contains non-existent 'dragonfruit': {}",
        dict.contains("dragonfruit")
    );

    println!("\n[5] Single Fuzzy Search (query: 'baxana' with distance 2 & transposition):");
    let single_results = dict
        .search("baxana")
        .distance(2)
        .transposition(true)
        .prefix(false)
        .limit(3)
        .execute()?;

    for result in &single_results {
        println!(
            "    • Result: {} | exact: {} | key: '{}' | dist: {}",
            result,
            result.is_exact(),
            result.key,
            result.distance
        );
    }

    let resultz = SearchResult {
        key: "apple".into(),
        distance: 1,
    };
    let score = resultz.similarity("appl");
    println!("{score}");

    println!("\n[6] Fuzzy Search with Lexicographical Range Bounds:");
    println!("    Searching 'pech' bounded strictly between 'p' and 'ph':");

    let bounded_results = dict
        .search("pech")
        .distance(1)
        .ge("p")
        .lt("ph")
        .gt("a")
        .le("z")
        .execute()?;

    for res in bounded_results {
        println!("    • Found: {}", res);
    }

    println!("\n[7] Testing embedded dictionary initialization (from_embedded):");

    let fst_bytes: &'static [u8] = Box::leak(fs::read(&fst_file_path)?.into_boxed_slice());
    let embedded_dict = Dictionary::from_embedded(fst_bytes)?;
    println!(
        "    ✓ Embedded dictionary loaded. Total keys: {}",
        embedded_dict.len()
    );

    println!("\n[8] Parallel Batch Fuzzy Search (Rayon):");
    let queries = ["aple", "baxana", "cheeriy", "pech"];

    let batch_results = dict
        .batch_search(&queries)
        .distance(2)
        .transposition(true)
        .prefix(true)
        .limit(2)
        .ge(b"a")
        .le(b"z")
        .execute();

    for (query, res) in queries.iter().zip(batch_results) {
        match res {
            Ok(matches) => {
                let match_str: Vec<String> = matches.iter().map(|m| m.key.clone()).collect();
                println!("    • Query '{}' => {:?}", query, match_str);
            }
            Err(err) => eprintln!("    • Query '{}' failed: {}", query, err),
        }
    }

    let _ = fs::remove_file(raw_file_path);
    let _ = fs::remove_file(fst_file_path);
    Ok(())
}
