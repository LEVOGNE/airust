#![forbid(unsafe_code)]
use airust::{KnowledgeBase, TrainingExample};
use sha2::{Digest, Sha256};
use std::{error::Error, hint::black_box, time::Instant};
fn main() -> Result<(), Box<dyn Error>> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let records: usize = a[0].parse()?;
    let width: usize = a[1].parse()?;
    let clones: usize = a[2].parse()?;
    let edits: usize = a[3].parse()?;
    assert!(
        (1..=8000).contains(&records)
            && (32..=4096).contains(&width)
            && clones <= 32
            && edits <= clones
    );
    let start = Instant::now();
    let examples = (0..records)
        .map(|i| TrainingExample {
            input: format!("{i}: Straße 東京 {}", "x".repeat(width)),
            output: format!("answer {i} {}", "y".repeat(width / 4)).into(),
            weight: 1.0,
            metadata: Some(serde_json::json!({"id":i,"scope":"training"})),
        })
        .collect();
    let base = KnowledgeBase::from_examples(examples);
    let build_ns = start.elapsed().as_nanos();
    let start = Instant::now();
    let mut snapshots: Vec<_> = (0..clones).map(|_| black_box(base.clone())).collect();
    let clone_ns = start.elapsed().as_nanos();
    let start = Instant::now();
    for (i, snapshot) in snapshots.iter_mut().enumerate().take(edits) {
        snapshot.remove_example(i % records)?;
        snapshot.add_example(format!("edit {i}"), "new", 2.0);
    }
    let edit_ns = start.elapsed().as_nanos();
    let mut hash = Sha256::new();
    for kb in std::iter::once(&base).chain(snapshots.iter()) {
        hash.update((kb.get_examples().len() as u64).to_le_bytes());
        for example in kb.get_examples() {
            hash.update(serde_json::to_vec(example)?);
        }
    }
    assert_eq!(base.get_examples().len(), records);
    assert!(base.get_examples()[0].input.starts_with("0:"));
    println!(
        "{}",
        serde_json::json!({"records":records,"width":width,"clones":clones,"edits":edits,"build_ns":build_ns,"clone_ns":clone_ns,"edit_ns":edit_ns,"signature_sha256":format!("{:x}",hash.finalize()),"library_version":airust::VERSION})
    );
    black_box(snapshots);
    black_box(base);
    Ok(())
}
