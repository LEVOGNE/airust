// src/bin/pdf2kb.rs - CLI tool for converting PDFs to Knowledge Bases
use airust::pdf_loader::{PdfLoader, PdfLoaderConfig};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn print_help() {
    println!("pdf2kb - PDF to Knowledge Base Converter");
    println!();
    println!("Usage:");
    println!("  pdf2kb <pdf-file>              - Converts PDF to a Knowledge Base JSON file in the knowledge/ folder");
    println!("  pdf2kb <pdf-file> [output]     - Converts PDF and saves the JSON file to the specified path");
    println!("  pdf2kb --help                  - Shows this help");
    println!();
    println!("Options:");
    println!("  --min-chunk <number>   - Minimum chunk size (default: 50)");
    println!("  --max-chunk <number>   - Maximum chunk size (default: 1000)");
    println!("  --overlap <number>     - Overlap between chunks (default: 200)");
    println!("  --weight <number>      - Weight for training examples (default: 1.0)");
    println!("  --no-metadata          - No metadata in training examples");
    println!("  --no-sentence-split    - Don't split text at sentence boundaries");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_help();
        return;
    }

    // Create default configuration
    let mut config = PdfLoaderConfig::default();
    let mut pdf_path = String::new();
    let mut output_path = String::new();

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--min-chunk" => {
                if i + 1 < args.len() {
                    if let Ok(value) = args[i + 1].parse::<usize>() {
                        config.min_chunk_size = value;
                    } else {
                        eprintln!("Error: --min-chunk requires a number");
                        process::exit(1);
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --min-chunk requires a value");
                    process::exit(1);
                }
            }
            "--max-chunk" => {
                if i + 1 < args.len() {
                    if let Ok(value) = args[i + 1].parse::<usize>() {
                        config.max_chunk_size = value;
                    } else {
                        eprintln!("Error: --max-chunk requires a number");
                        process::exit(1);
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --max-chunk requires a value");
                    process::exit(1);
                }
            }
            "--overlap" => {
                if i + 1 < args.len() {
                    if let Ok(value) = args[i + 1].parse::<usize>() {
                        config.chunk_overlap = value;
                    } else {
                        eprintln!("Error: --overlap requires a number");
                        process::exit(1);
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --overlap requires a value");
                    process::exit(1);
                }
            }
            "--weight" => {
                if i + 1 < args.len() {
                    if let Ok(value) = args[i + 1].parse::<f32>() {
                        config.default_weight = value;
                    } else {
                        eprintln!("Error: --weight requires a decimal number");
                        process::exit(1);
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --weight requires a value");
                    process::exit(1);
                }
            }
            "--no-metadata" => {
                config.include_metadata = false;
                i += 1;
            }
            "--no-sentence-split" => {
                config.split_by_sentence = false;
                i += 1;
            }
            _ => {
                // If it's not an option, it should be a file path
                if pdf_path.is_empty() {
                    pdf_path = args[i].clone();
                } else if output_path.is_empty() {
                    output_path = args[i].clone();
                } else {
                    eprintln!("Unknown argument: {}", args[i]);
                    print_help();
                    process::exit(1);
                }
                i += 1;
            }
        }
    }

    // Check if PDF path was specified
    if pdf_path.is_empty() {
        eprintln!("Error: No PDF file specified");
        print_help();
        process::exit(1);
    }

    // If no output path was specified, use the PDF name and save in the knowledge/ folder
    if output_path.is_empty() {
        let pdf_path_buf = PathBuf::from(&pdf_path);
        let file_stem = pdf_path_buf.file_stem().unwrap_or_default();

        // Create knowledge/ directory if it doesn't exist
        let knowledge_dir = PathBuf::from("knowledge");
        if !knowledge_dir.exists() {
            match fs::create_dir_all(&knowledge_dir) {
                Ok(_) => println!("Directory 'knowledge/' created."),
                Err(e) => {
                    eprintln!("Error creating 'knowledge/' directory: {}", e);
                    process::exit(1);
                }
            }
        }

        // Create the output path in the knowledge/ directory
        let mut output_path_buf = knowledge_dir.clone();
        output_path_buf.push(file_stem);
        output_path_buf.set_extension("json");
        output_path = output_path_buf.to_string_lossy().to_string();
    }

    println!("Converting PDF: {}", pdf_path);
    println!("Output: {}", output_path);
    println!("Configuration:");
    println!("  Min. chunk size: {} characters", config.min_chunk_size);
    println!("  Max. chunk size: {} characters", config.max_chunk_size);
    println!("  Chunk overlap: {} characters", config.chunk_overlap);
    println!("  Weight: {}", config.default_weight);
    println!(
        "  Metadata: {}",
        if config.include_metadata { "Yes" } else { "No" }
    );
    println!(
        "  Sentence boundary split: {}",
        if config.split_by_sentence {
            "Yes"
        } else {
            "No"
        }
    );

    // Convert PDF to Knowledge Base
    let loader = PdfLoader::with_config(config);
    match loader.pdf_to_knowledge_base(&pdf_path) {
        Ok(kb) => {
            println!(
                "PDF successfully converted. {} training examples extracted.",
                kb.get_examples().len()
            );

            // Save Knowledge Base
            match loader.save_knowledge_base(&kb, &output_path) {
                Ok(_) => {
                    println!("✓ Knowledge Base successfully saved: {}", output_path);
                    println!("\nTest the base with: cargo run --bin airust -- query tfidf \"Question about the content\"");
                }
                Err(e) => {
                    eprintln!("❌ Error saving the Knowledge Base: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error converting the PDF: {}", e);
            process::exit(1);
        }
    }
}
