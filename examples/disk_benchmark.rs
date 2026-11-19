use airust::{disk_index::*, storage::StoreChange, Engine, KnowledgeEntry};
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n = std::env::args()
        .nth(1)
        .unwrap_or("10000".into())
        .parse::<usize>()?;
    if !(1000..=1000000).contains(&n) {
        return Err("size must be 1000..1000000".into());
    }
    let mode = std::env::args().nth(2).unwrap_or("disk".into());
    if mode == "memory" {
        let started = Instant::now();
        let engine = Engine::from_entries((0..n).map(record))?;
        let build_ms = started.elapsed().as_secs_f64() * 1000.;
        let started = Instant::now();
        let hits = engine.search("category42", 10);
        let query_ms = started.elapsed().as_secs_f64() * 1000.;
        assert_eq!(hits.len(), 10);
        println!(
            "{}",
            serde_json::json!({"schema":1,"mode":mode,"fixture":"identical synthetic records; native scoring recipes differ","entries":engine.len(),"build_ms":build_ms,"query_ms":query_ms,"ids":hits.iter().map(|h|&h.entry.id).collect::<Vec<_>>()})
        );
        return Ok(());
    }
    if mode != "disk" {
        return Err("mode must be memory or disk".into());
    }
    let folder = tempfile::tempdir()?;
    let path = folder.path().join("index.sqlite");
    let mut db = DiskIndex::open(&path, "benchmark", DiskOptions::default())?;
    let start = Instant::now();
    for batch in 0..n.div_ceil(1000) {
        let changes = (batch * 1000..((batch + 1) * 1000).min(n))
            .map(|i| StoreChange::Put(record(i)))
            .collect::<Vec<_>>();
        db.apply(&db.revision()?, &changes, &DiskWork::default())?;
    }
    let build_ms = start.elapsed().as_secs_f64() * 1000.;
    drop(db);
    let start = Instant::now();
    let mut db = DiskIndex::open(&path, "benchmark", DiskOptions::default())?;
    let open_ms = start.elapsed().as_secs_f64() * 1000.;
    let start = Instant::now();
    let hits = db.search("category42", &DiskSearchOptions::new(10)?)?;
    let query_ms = start.elapsed().as_secs_f64() * 1000.;
    assert_eq!(hits.hits().len(), 10);
    db.integrity_check(&DiskWork::default())?;
    println!(
        "{}",
        serde_json::json!({"schema":1,"mode":mode,"fixture":"bounded public synthetic disk-index workload, not application capacity","entries":db.len()?,"build_ms":build_ms,"reopen_ms":open_ms,"query_ms":query_ms,"file_bytes":std::fs::metadata(&path)?.len(),"hits":hits.hits().len(),"ids":hits.hits().iter().map(|h|&h.entry.id).collect::<Vec<_>>(),"recipe":db.recipe()})
    );
    Ok(())
}

fn record(i: usize) -> KnowledgeEntry {
    KnowledgeEntry::new(
        format!("id-{i:08}"),
        format!("project knowledge category{} record{i}", i % 100),
    )
}
