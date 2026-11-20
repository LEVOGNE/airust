use airust::pdf_loader::{PdfImportLimits, PdfLoader, PdfLoaderConfig};
use std::{env, fs, path::PathBuf};

fn option_value(
    args: &mut impl Iterator<Item = std::ffi::OsString>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    args.next()
        .ok_or_else(|| format!("{flag} requires a value"))?
        .into_string()
        .map_err(|_| format!("{flag} requires a valid Unicode value").into())
}

fn main() {
    if let Err(error) = run() {
        eprintln!(
            "PDF conversion failed: {}",
            terminal_text(&error.to_string())
        );
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
    println!("--max-file-mib N    Bound actual input bytes (default in bounded mode: 256 MiB)");
    println!("--max-pages N       Bound physical pages (default in bounded mode: 20000)");
    println!(
        "--max-text-mib N    Bound raw, cleaned and chunk text bytes separately (default: 64 MiB)"
    );
    println!("--max-chunks N      Bound chunks across all pages (default in bounded mode: 100000)");
    println!("Limits are opt-in. Selecting any limit enables the other bounded-mode defaults.");
    println!("--                  End options; remaining arguments are file paths");
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = PdfLoaderConfig::default();
    let mut paths = Vec::new();
    let mut verbose = false;
    let mut overwrite = false;
    let mut positional_only = false;
    let defaults = PdfImportLimits::default();
    let mut bounded = false;
    let mut max_file_bytes = defaults.max_file_bytes();
    let mut max_pages = defaults.max_pages();
    let mut max_text_bytes = defaults.max_text_bytes();
    let mut max_chunks = defaults.max_chunks();
    let mut args = env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if positional_only {
            paths.push(PathBuf::from(arg));
            continue;
        }
        let arg = match arg.into_string() {
            Ok(arg) => arg,
            Err(arg) => {
                if arg.to_string_lossy().starts_with('-') {
                    return Err(format!("Unknown option: {}", arg.to_string_lossy()).into());
                }
                paths.push(PathBuf::from(arg));
                continue;
            }
        };
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
                config.min_chunk_size = option_value(&mut args, "--min-chunk")?.parse()?
            }
            "--max-chunk" => {
                config.max_chunk_size = option_value(&mut args, "--max-chunk")?.parse()?
            }
            "--overlap" => config.chunk_overlap = option_value(&mut args, "--overlap")?.parse()?,
            "--weight" => config.default_weight = option_value(&mut args, "--weight")?.parse()?,
            "--no-metadata" => config.include_metadata = false,
            "--no-sentence-split" => config.split_by_sentence = false,
            "--paragraph-split" => config.split_by_paragraph = true,
            "--no-paragraph-split" => config.split_by_paragraph = false,
            "--no-clean" => config.clean_text = false,
            "--no-dehyphenate" => config.dehyphenate = false,
            "--keep-page-numbers" => config.remove_page_numbers = false,
            "--verbose" => verbose = true,
            "--overwrite" => overwrite = true,
            "--max-file-mib" => {
                max_file_bytes = option_value(&mut args, "--max-file-mib")?
                    .parse::<u64>()?
                    .checked_mul(1024 * 1024)
                    .ok_or("input byte limit overflow")?;
                bounded = true;
            }
            "--max-pages" => {
                max_pages = option_value(&mut args, "--max-pages")?.parse()?;
                bounded = true;
            }
            "--max-text-mib" => {
                max_text_bytes = option_value(&mut args, "--max-text-mib")?
                    .parse::<usize>()?
                    .checked_mul(1024 * 1024)
                    .ok_or("text byte limit overflow")?;
                bounded = true;
            }
            "--max-chunks" => {
                max_chunks = option_value(&mut args, "--max-chunks")?.parse()?;
                bounded = true;
            }
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
    let limits = if bounded {
        Some(PdfImportLimits::new(
            max_file_bytes,
            max_pages,
            max_text_bytes,
            max_chunks,
        )?)
    } else {
        None
    };
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
        let raw_loader = PdfLoader::with_config(raw_config);
        let raw_loader = if let Some(limits) = limits {
            raw_loader.with_limits(limits)
        } else {
            raw_loader
        };
        let pages = raw_loader.extract_pages_from_pdf(input)?;
        println!("Raw preview:");
        for page in pages.iter().take(3) {
            println!("Page {}: {}", page.page_number, preview(&page.text, 300));
        }
    }
    let loader = PdfLoader::with_config(config);
    let loader = if let Some(limits) = limits {
        loader.with_limits(limits)
    } else {
        loader
    };
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
        terminal_text(&output.to_string_lossy())
    );
    Ok(())
}

fn preview(text: &str, limit: usize) -> String {
    terminal_text(&text.chars().take(limit).collect::<String>())
}

fn terminal_text(text: &str) -> String {
    let mut output = String::new();
    for character in text.chars() {
        if character.is_control()
            || matches!(character, '\u{061c}'|'\u{200e}'|'\u{200f}'|'\u{2028}'..='\u{202e}'|'\u{2066}'..='\u{2069}')
        {
            output.extend(character.escape_default());
        } else {
            output.push(character);
        }
    }
    output
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

    #[test]
    fn console_previews_escape_controls_without_changing_readable_unicode() {
        let text = "Rust\u{1b}[31m\nä😀\u{202e}";
        let result = preview(text, 100);
        assert!(!result.contains('\u{1b}'));
        assert!(!result.contains('\n'));
        assert!(!result.contains('\u{202e}'));
        assert!(result.contains("ä😀"));
        assert!(result.contains("\\u{1b}"));
    }
}
