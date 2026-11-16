use airust::KnowledgeBase;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("Merge failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = PathBuf::from("knowledge/train.json");
    let mut inputs = Vec::new();
    let mut include_embedded = false;
    let mut positional_only = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if positional_only {
            inputs.push(PathBuf::from(arg));
            continue;
        }
        match arg.as_str() {
            "--help" | "-h" => {
                println!("merge_kb [SOURCE.json ...] [--output TARGET.json] [--include-embedded]");
                println!("Without sources: knowledge/*.json excluding the destination. Exact duplicates are removed.");
                return Ok(());
            }
            "--output" => {
                let value = args
                    .next()
                    .filter(|value| !value.starts_with('-'))
                    .ok_or("--output requires a path")?;
                output = value.into();
            }
            "--" => positional_only = true,
            "--include-embedded" => include_embedded = true,
            value if value.starts_with('-') => {
                return Err(format!("Unknown option: {value}").into())
            }
            value => inputs.push(PathBuf::from(value)),
        }
    }
    if inputs.is_empty() {
        match fs::read_dir("knowledge") {
            Ok(entries) => {
                for entry in entries {
                    let path = entry?.path();
                    if path.is_file()
                        && path.extension().is_some_and(|ext| ext == "json")
                        && !same_path(&path, &output)
                    {
                        inputs.push(path);
                    }
                }
            }
            Err(error) if include_embedded && error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    inputs.sort();
    inputs.dedup();
    if inputs.is_empty() && !include_embedded {
        return Err("No source files found. The output file is never read as a source.".into());
    }
    let mut examples = Vec::new();
    let mut seen = BTreeSet::new();
    for path in &inputs {
        if same_path(path, &output) {
            return Err(format!(
                "Source and destination are the same file: {}",
                path.display()
            )
            .into());
        }

        let kb = KnowledgeBase::load(path.clone())?;
        for example in kb.get_examples() {
            if seen.insert(serde_json::to_string(example)?) {
                examples.push(example.clone());
            }
        }
    }
    if include_embedded {
        for example in KnowledgeBase::from_embedded().get_examples() {
            if seen.insert(serde_json::to_string(example)?) {
                examples.push(example.clone());
            }
        }
    }
    let count = examples.len();
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    KnowledgeBase::from_examples(examples).save(Some(output.clone()))?;
    println!(
        "{} sources, {} unique examples -> {}",
        inputs.len(),
        count,
        output.display()
    );
    Ok(())
}

fn same_path(a: &Path, b: &Path) -> bool {
    a == b
        || a.canonicalize()
            .ok()
            .zip(b.canonicalize().ok())
            .is_some_and(|(a, b)| a == b)
}
