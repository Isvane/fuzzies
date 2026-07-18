// How to run:
// cargo run --release --example search

use fuzzies::Dictionary;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let txt_path = "words.txt";
    let fst_path = "words.fst";

    // Setup dummy data (out of order to test sorting)
    {
        let mut file = File::create(txt_path)?;
        writeln!(file, "banana")?;
        writeln!(file, "apple")?;
        writeln!(file, "apricot")?;
        writeln!(file, "cherry")?;
        writeln!(file, "blueberry")?;
    }

    // FSTs require sorted bytes, so we sort and build the binary file
    Dictionary::sort(txt_path)?;
    Dictionary::build(txt_path, fst_path)?;

    // Load up our compiled dictionary via mmap
    let dict = Dictionary::open(fst_path)?;

    // Typo correction (distance = 1)
    println!("--- Searching for 'appl' ---");
    let hits = dict.search("appl").execute()?;
    for hit in hits {
        println!("Match: {} (Distance: {})", hit.key, hit.distance);
    }

    // Transposition swap (e.g., matching 'banana' from 'abnana')
    println!("\n--- Searching for 'abnana' (with transposition) ---");
    let hits = dict
        .search("abnana")
        .distance(1)
        .transposition(true)
        .execute()?;
    for hit in hits {
        println!("Match: {} (Distance: {})", hit.key, hit.distance);
    }

    // Prefix fuzzy match (matches things starting like the query)
    println!("\n--- Prefix search for 'blue' ---");
    let hits = dict.search("blue").prefix(true).execute()?;
    for hit in hits {
        println!("Match: {} (Distance: {})", hit.key, hit.distance);
    }

    // Concurrent Batch Queries
    println!("\n--- Batch Search ---");
    let queries = vec!["aple", "cheriy"];
    let batch_hits = dict.batch_search(&queries).execute();

    for (query, result) in queries.iter().zip(batch_hits) {
        if let Ok(matches) = result {
            println!("Query '{}' matches: {:?}", query, matches);
        }
    }

    // Clean up temporary files
    std::fs::remove_file(txt_path)?;
    std::fs::remove_file(fst_path)?;

    Ok(())
}
