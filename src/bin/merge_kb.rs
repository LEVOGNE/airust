// src/bin/merge_kb.rs - Tool for merging all JSON files in the knowledge/ directory
use airust::knowledge::KnowledgeBase;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    println!("=== Knowledge Base Merger ===");
    println!("Searching the knowledge/ directory for JSON files...");

    // Path to the knowledge directory
    let knowledge_dir = PathBuf::from("knowledge");

    // Check if the directory exists
    if !knowledge_dir.exists() {
        eprintln!("Error: The 'knowledge/' directory does not exist!");
        process::exit(1);
    }

    // Find all JSON files in the directory
    let mut json_files = Vec::new();
    match fs::read_dir(&knowledge_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                        json_files.push(path);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading directory: {}", e);
            process::exit(1);
        }
    }

    // Check if JSON files were found
    if json_files.is_empty() {
        eprintln!("No JSON files found in the knowledge/ directory!");
        process::exit(1);
    }

    println!("{} JSON files found:", json_files.len());
    for file in &json_files {
        println!("  - {}", file.file_name().unwrap().to_string_lossy());
    }

    // Create target Knowledge Base
    let mut merged_kb = KnowledgeBase::new();

    // Load and merge all JSON files
    for file in &json_files {
        match KnowledgeBase::load(file.clone()) {
            Ok(kb) => {
                let example_count = kb.get_examples().len();
                println!(
                    "File {} loaded: {} examples",
                    file.file_name().unwrap().to_string_lossy(),
                    example_count
                );
                merged_kb.merge(&kb);
            }
            Err(e) => {
                println!(
                    "Warning: Could not load file {}: {}",
                    file.file_name().unwrap().to_string_lossy(),
                    e
                );
            }
        }
    }

    println!(
        "\nTotal Knowledge Base contains {} examples.",
        merged_kb.get_examples().len()
    );

    // Ask if the standard examples should be included
    println!("\nDo you want to add the standard examples (embedded data)? (y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase() == "y" {
        merged_kb.merge_embedded();
        println!(
            "Standard examples added. New size: {} examples.",
            merged_kb.get_examples().len()
        );
    }

    // Save Knowledge Base in the knowledge/ directory
    let out_path = knowledge_dir.join("train.json");
    match merged_kb.save(Some(out_path.clone())) {
        Ok(_) => {
            println!("✓ Merged Knowledge Base successfully saved as 'knowledge/train.json'!");
            println!("\nYou can now use it with standard AIRust commands:");
            println!("  cargo run --bin airust -- query tfidf \"Your question here\"");
            println!("  cargo run --bin airust -- interactive");
        }
        Err(e) => {
            eprintln!("❌ Error saving the Knowledge Base: {}", e);
        }
    }
}
