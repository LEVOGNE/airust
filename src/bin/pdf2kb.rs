use airust::pdf_loader::{PdfLoader, PdfLoaderConfig};
use std::{env, fs, path::PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("PDF conversion failed: {error}");
        std::process::exit(1);
    }
}

fn print_help() {
    println!("pdf2kb INPUT.pdf [OUTPUT.json] [OPTIONS]");
    println!("Default output: knowledge/<input-name>.json");
    println!("--min-chunk N        Preferred minimum character count (default 50)");
    println!("--max-chunk N        Maximum character count (default 1000)");
    println!("--overlap N          Overlap character count (default 200)");
    println!("--weight N           Finite nonnegative example weight (default 1)");
    println!("--no-metadata        Omit import provenance");
    println!("--no-sentence-split  Disable sentence boundary splitting");
    println!("--paragraph-split    Prefer paragraph boundaries (default)");
    println!("--no-paragraph-split Disable paragraph boundary splitting");
    println!("--no-clean           Disable text cleaning");
    println!("--no-dehyphenate     Keep hyphenated line breaks");
    println!("--keep-page-numbers  Keep standalone page-number lines");
    println!("--verbose           Show bounded text previews");
    println!("--overwrite         Explicitly replace an existing output file");
    println!("--                  End options; remaining arguments are file paths");
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = PdfLoaderConfig::default();
    let mut paths = Vec::new();
    let mut verbose = false;
    let mut overwrite = false;
    let mut positional_only = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if positional_only {
            paths.push(PathBuf::from(arg));
            continue;
        }
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--version" | "-V" => {
                println!("pdf2kb {}", airust::VERSION);
                return Ok(());
            }
            "--min-chunk" => {
                config.min_chunk_size =
                    args.next().ok_or("--min-chunk requires a value")?.parse()?
            }
            "--max-chunk" => {
                config.max_chunk_size =
                    args.next().ok_or("--max-chunk requires a value")?.parse()?
            }
            "--overlap" => {
                config.chunk_overlap = args.next().ok_or("--overlap requires a value")?.parse()?
            }
            "--weight" => {
                config.default_weight = args.next().ok_or("--weight requires a value")?.parse()?
            }
            "--no-metadata" => config.include_metadata = false,
            "--no-sentence-split" => config.split_by_sentence = false,
            "--paragraph-split" => config.split_by_paragraph = true,
            "--no-paragraph-split" => config.split_by_paragraph = false,
            "--no-clean" => config.clean_text = false,
            "--no-dehyphenate" => config.dehyphenate = false,
            "--keep-page-numbers" => config.remove_page_numbers = false,
            "--verbose" => verbose = true,
            "--overwrite" => overwrite = true,
            "--" => positional_only = true,
            value if value.starts_with('-') => {
                return Err(format!("Unknown option: {value}").into())
            }
            value => paths.push(PathBuf::from(value)),
        }
    }
    if paths.is_empty() {
        return Err("A PDF input path is required; use --help for usage".into());
    }
    if paths.len() > 2 {
        return Err("Expected one input path and at most one output path".into());
    }
    config.validate()?;
    let input = &paths[0];
    if !input.is_file() {
        return Err(format!("Input is not a regular file: {}", input.display()).into());
    }
    let output = paths.get(1).cloned().unwrap_or_else(|| {
        PathBuf::from("knowledge")
            .join(input.file_stem().unwrap_or_default())
            .with_extension("json")
    });
    if input == &output
        || input
            .canonicalize()
            .ok()
            .zip(output.canonicalize().ok())
            .is_some_and(|(a, b)| a == b)
    {
        return Err("Input and output must be different files".into());
    }
    if output.exists() && !overwrite {
        return Err("Output already exists; pass --overwrite to replace it".into());
    }
    if verbose {
        let mut raw_config = config.clone();
        raw_config.clean_text = false;
        let pages = PdfLoader::with_config(raw_config).extract_pages_from_pdf(input)?;
        println!("Raw preview:");
        for page in pages.iter().take(3) {
            println!("Page {}: {}", page.page_number, preview(&page.text, 300));
        }
    }
    let loader = PdfLoader::with_config(config);
    let knowledge = loader.pdf_to_knowledge_base(input)?;
    if verbose {
        println!("Cleaned preview:");
        for (index, example) in knowledge.get_examples().iter().take(5).enumerate() {
            println!("Chunk {index}: {}", preview(&example.input, 300));
        }
    }
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    if overwrite {
        loader.save_knowledge_base(&knowledge, &output)?;
    } else {
        loader.save_knowledge_base_if_absent(&knowledge, &output)?;
    }
    println!(
        "Saved {} training examples to {}",
        knowledge.get_examples().len(),
        output.display()
    );
    Ok(())
}

fn preview(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn previews_do_not_split_utf8_characters() {
        let text = format!("{}尾", "ä😀".repeat(200));
        assert_eq!(preview(&text, 300).chars().count(), 300);
        assert_eq!(preview(&text, usize::MAX), text);
    }
}
