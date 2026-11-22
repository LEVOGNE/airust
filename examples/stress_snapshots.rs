use airust::{execution::*, provenance::SourceReference, snapshots::*, KnowledgeEntry};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Barrier,
    },
    time::{Duration, Instant},
};
fn state(generation: u64) -> DatasetRecord {
    DatasetRecord::new(KnowledgeEntry::new(
        "state",
        format!("generation {generation}"),
    ))
    .with_sources(vec![SourceReference::new("state-source")
        .unwrap()
        .with_revision(generation.to_string())
        .unwrap()])
}
fn summary(mut values: Vec<f64>) -> Value {
    values.sort_by(f64::total_cmp);
    let at = |f: f64| values[((values.len() - 1) as f64 * f).ceil() as usize];
    json!({"samples":values.len(),"p50_ms":at(0.5),"p95_ms":at(0.95),"p99_ms":at(0.99),"max_ms":values.last()})
}
fn try_generation(
    manager: &DatasetManager,
    base: &DatasetSnapshot,
    control: &WorkControl,
    prepare_ms: &mut Vec<f64>,
    publish_ms: &mut Vec<f64>,
) -> Result<(), DatasetError> {
    let start = Instant::now();
    let prepared = manager.prepare_controlled(
        base.revision(),
        DatasetUpdate::new(vec![DatasetChange::Put(state(
            base.revision().generation() + 1,
        ))]),
        None,
        control,
    );
    prepare_ms.push(start.elapsed().as_secs_f64() * 1000.0);
    let prepared = prepared?;
    let start = Instant::now();
    let published = manager.publish_controlled(prepared, control);
    publish_ms.push(start.elapsed().as_secs_f64() * 1000.0);
    published.map(|_| ())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = std::env::args()
        .nth(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(1000usize);
    if !(1000..=100000).contains(&count) {
        return Err("stress_snapshots needs 1000..100000 records".into());
    }
    let manager = DatasetManager::new("stress", DatasetConfig::default())?;
    let control = WorkControl::default().with_timeout(Duration::from_secs(120))?;
    let empty = manager.snapshot()?;
    let records = (1..count)
        .map(|i| {
            DatasetRecord::new(KnowledgeEntry::new(
                format!("item{i}"),
                format!("Rust ownership topic{}", i % 1000),
            ))
        })
        .chain([state(1)])
        .collect();
    let start = Instant::now();
    let initial = manager.publish_controlled(
        manager.prepare_replace_controlled(empty.revision(), records, None, None, &control)?,
        &control,
    )?;
    let build_ms = start.elapsed().as_secs_f64() * 1000.0;
    let barrier = Arc::new(Barrier::new(6));
    let done = Arc::new(AtomicUsize::new(0));
    let conflicts = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let (reader_rows, writer_rows) = std::thread::scope(|scope| {
        let mut readers = Vec::new();
        let mut writers = Vec::new();
        for worker in 0..4 {
            let manager = manager.clone();
            let barrier = barrier.clone();
            let done = done.clone();
            let control = control.clone();
            readers.push(scope.spawn(move || {
                let mut times = Vec::new();
                let mut generations = BTreeSet::new();
                barrier.wait();
                while done.load(Ordering::Acquire) < 2 || times.len() < 100 {
                    control.checkpoint().unwrap();
                    assert!(times.len() < 100000, "reader operation guard exceeded");
                    let start = Instant::now();
                    let view = manager.snapshot().unwrap();
                    let generation = view.revision().generation();
                    generations.insert(generation);
                    assert_eq!(
                        view.lexical().get("state").unwrap().text,
                        format!("generation {generation}")
                    );
                    assert_eq!(
                        view.provenance()
                            .sources_for("state")
                            .next()
                            .unwrap()
                            .revision(),
                        Some(generation.to_string().as_str())
                    );
                    let hits = view
                        .lexical()
                        .search_controlled(
                            &format!("topic{}", worker * 17),
                            &airust::SearchOptions::default(),
                            &control,
                        )
                        .unwrap();
                    for hit in hits {
                        assert!(view.lexical().get(&hit.entry.id).is_some());
                    }
                    times.push(start.elapsed().as_secs_f64() * 1000.0);
                }
                json!({"worker":worker,"generations":generations,"latency":summary(times)})
            }));
        }
        for worker in 0..2 {
            let manager = manager.clone();
            let barrier = barrier.clone();
            let done = done.clone();
            let conflicts = conflicts.clone();
            let control = control.clone();
            writers.push(scope.spawn(move || {
                let mut prepare_ms = Vec::new();
                let mut publish_ms = Vec::new();
                let mut completed = 0;
                barrier.wait();
                while completed < 5 {
                    control.checkpoint().unwrap();
                    let base = manager.snapshot().unwrap();
                    match try_generation(
                        &manager,
                        &base,
                        &control,
                        &mut prepare_ms,
                        &mut publish_ms,
                    ) {
                        Ok(()) => completed += 1,
                        Err(DatasetError::Conflict) => {
                            assert!(
                                conflicts.fetch_add(1, Ordering::Relaxed) < 1000,
                                "conflict guard exceeded"
                            );
                        }
                        Err(error) => panic!("{error}"),
                    }
                }
                done.fetch_add(1, Ordering::Release);
                json!({"worker":worker,"commits":completed,"prepare":summary(prepare_ms),"publish":summary(publish_ms)})
            }));
        }
        (
            readers
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>(),
            writers
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>(),
        )
    });
    let current = manager.snapshot()?;
    assert_eq!(current.revision().generation(), 11);
    assert_eq!(current.lexical().len(), count);
    assert_eq!(initial.lexical().get("state").unwrap().text, "generation 1");
    assert_eq!(
        initial
            .provenance()
            .sources_for("state")
            .next()
            .unwrap()
            .revision(),
        Some("1")
    );
    println!(
        "{}",
        json!({"version":airust::VERSION,"fixture":"stress-snapshots-v1","entries":count,"build_ms":build_ms,"concurrent_elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"readers":reader_rows,"writers":writer_rows,"conflicts":conflicts.load(Ordering::Relaxed),"final_generation":current.revision().generation(),"old_view_unchanged":true,"limits":"Synthetic lexical/source snapshots; 4 readers and 2 competing writers, 10 total commits; no vector model or external storage transaction."})
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_preparation_is_retryable_before_publication() {
        let manager = DatasetManager::new("example-retry", DatasetConfig::default()).unwrap();
        let control = WorkControl::default();
        let empty = manager.snapshot().unwrap();
        let mut prepare = Vec::new();
        let mut publish = Vec::new();
        try_generation(&manager, &empty, &control, &mut prepare, &mut publish).unwrap();
        assert!(matches!(
            try_generation(&manager, &empty, &control, &mut prepare, &mut publish),
            Err(DatasetError::Conflict)
        ));
        assert_eq!(prepare.len(), 2);
        assert_eq!(publish.len(), 1);
        let fresh = manager.snapshot().unwrap();
        assert_eq!(fresh.revision().generation(), 1);
        assert_eq!(fresh.lexical().get("state").unwrap().text, "generation 1");
        try_generation(&manager, &fresh, &control, &mut prepare, &mut publish).unwrap();
        assert_eq!(manager.snapshot().unwrap().revision().generation(), 2);
        assert!(empty.lexical().is_empty());
        assert_eq!(fresh.lexical().get("state").unwrap().text, "generation 1");
    }
}
