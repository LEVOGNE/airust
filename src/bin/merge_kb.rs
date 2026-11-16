//! Merge legacy training JSON without reading the destination back into itself.
use airust::KnowledgeBase;
use std::collections::BTreeSet;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("Zusammenfügen fehlgeschlagen: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = PathBuf::from("knowledge/train.json");
    let mut inputs = Vec::new();
    let mut include_embedded = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("merge_kb [QUELLE.json ...] [--output ZIEL.json] [--include-embedded]");
                println!("Ohne Quellen: knowledge/*.json, ohne die Zieldatei. Exakte Duplikate werden entfernt.");
                return Ok(());
            }
            "--output" => output = args.next().ok_or("--output benötigt einen Pfad")?.into(),
            "--include-embedded" => include_embedded = true,
            value if value.starts_with('-') => {
                return Err(format!("Unbekannte Option: {value}").into())
            }
            value => inputs.push(PathBuf::from(value)),
        }
    }
    if inputs.is_empty() {
        for entry in fs::read_dir("knowledge")? {
            let path = entry?.path();
            if path.is_file()
                && path.extension().is_some_and(|ext| ext == "json")
                && !same_path(&path, &output)
            {
                inputs.push(path);
            }
        }
    }
    inputs.sort();
    inputs.dedup();
    if inputs.is_empty() && !include_embedded {
        return Err(
            "Keine Quelldateien gefunden. Die Zieldatei wird nicht als Quelle eingelesen.".into(),
        );
    }
    let mut examples = Vec::new();
    let mut seen = BTreeSet::new();
    for path in &inputs {
        if same_path(path, &output) {
            return Err(format!("Quelle und Ziel sind dieselbe Datei: {}", path.display()).into());
        }
        // Read and validate every source before touching the output.
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
    let data = serde_json::to_string_pretty(&examples)?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let filename = output
        .file_name()
        .ok_or("Ziel benötigt einen Dateinamen")?
        .to_string_lossy();
    let temporary = parent.join(format!(".{filename}.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> std::io::Result<()> {
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, &output)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    println!(
        "{} Quellen, {} eindeutige Beispiele → {}",
        inputs.len(),
        examples.len(),
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
