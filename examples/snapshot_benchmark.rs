use airust::{
    provenance::SourceReference,
    semantic::{EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace},
    snapshots::{
        DatasetChange, DatasetConfig, DatasetError, DatasetManager, DatasetRecord, DatasetUpdate,
    },
    KnowledgeEntry,
};
use serde_json::json;
use std::{hint::black_box, time::Instant};
struct Provider {
    space: EmbeddingSpace,
    embedded: usize,
}
impl EmbeddingProvider for Provider {
    fn space(&self) -> &EmbeddingSpace {
        &self.space
    }
    fn embed(
        &mut self,
        texts: &[&str],
        _: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.embedded += texts.len();
        Ok(texts
            .iter()
            .map(|t| vec![1.0, (t.len() % 97 + 1) as f32])
            .collect())
    }
}
fn record(i: usize, revision: usize) -> Result<DatasetRecord, Box<dyn std::error::Error>> {
    Ok(DatasetRecord::new(KnowledgeEntry::new(
        format!("record-{i:08}"),
        format!("Rust knowledge record {i} at revision {revision}. Häuser und Gärten."),
    ))
    .with_sources(vec![SourceReference::new(format!("source-{i:08}"))?
        .with_revision(format!("revision-{revision}"))?]))
}
fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "10000".into())
        .parse::<usize>()?;
    if count == 0 || count > 100000 {
        return Err("record count must be 1..100000".into());
    }
    let positional = std::env::args().nth(2).as_deref() == Some("text");
    #[cfg(not(feature = "advanced-search"))]
    if positional {
        return Err("text projection requires advanced-search".into());
    }
    let mut provider = Provider {
        space: EmbeddingSpace::new("handwritten-snapshot-benchmark", 2)?,
        embedded: 0,
    };
    let config = DatasetConfig::default()
        .with_embedding_space(provider.space.clone())?
        .with_batch_size(128)?;
    #[cfg(feature = "advanced-search")]
    let config = if positional {
        config.with_text_analyzer(airust::text_search::AnalyzerConfig::german())
    } else {
        config
    };
    let manager = DatasetManager::new("benchmark", config)?;
    let empty = manager.snapshot()?;
    let records = (0..count)
        .map(|i| record(i, 1))
        .collect::<Result<Vec<_>, _>>()?;
    let start = Instant::now();
    let prepared = manager.prepare_replace(empty.revision(), records, None, Some(&mut provider))?;
    let initial_prepare_ms = ms(start);
    assert_eq!(provider.embedded, count);
    let start = Instant::now();
    let initial = manager.publish(prepared)?;
    let initial_publish_ms = ms(start);
    assert_eq!(initial.lexical().len(), count);
    assert_eq!(initial.provenance().entry_count(), count);
    let changed = 100.min(count);
    provider.embedded = 0;
    let changes = (0..changed)
        .map(|i| record(i, 2).map(DatasetChange::Put))
        .collect::<Result<Vec<_>, _>>()?;
    let start = Instant::now();
    let prepared = manager.prepare(
        initial.revision(),
        DatasetUpdate::new(changes),
        Some(&mut provider),
    )?;
    let changed_prepare_ms = ms(start);
    assert_eq!(provider.embedded, changed);
    assert_eq!(prepared.stats().reused_vectors, count - changed);
    let start = Instant::now();
    let current = manager.publish(prepared)?;
    let changed_publish_ms = ms(start);
    let changes = (0..changed)
        .map(|i| {
            let id = format!("record-{i:08}");
            let entry = current
                .lexical()
                .get(&id)
                .unwrap()
                .clone()
                .with_metadata(json!({"reviewed":true}));
            DatasetChange::Put(
                DatasetRecord::new(entry)
                    .with_sources(current.provenance().sources_for(&id).cloned().collect()),
            )
        })
        .collect();
    let start = Instant::now();
    let prepared = manager.prepare(current.revision(), DatasetUpdate::new(changes), None)?;
    let metadata_prepare_ms = ms(start);
    assert_eq!(prepared.stats().embedded_entries, 0);
    let metadata = manager.publish(prepared)?;
    let old_vector = initial
        .semantic()
        .unwrap()
        .embedding("record-00000000")
        .unwrap();
    assert_eq!(
        initial.lexical().get("record-00000000").unwrap().text,
        record(0, 1)?.entry().text
    );
    assert_eq!(old_vector.len(), 2);
    let deletes = (0..changed)
        .map(|i| DatasetChange::Delete(format!("record-{i:08}")))
        .collect();
    let start = Instant::now();
    let prepared = manager.prepare(metadata.revision(), DatasetUpdate::new(deletes), None)?;
    let deletion_prepare_ms = ms(start);
    let current = manager.publish(prepared)?;
    assert_eq!(current.lexical().len(), count - changed);
    assert!(current
        .semantic()
        .unwrap()
        .embedding("record-00000000")
        .is_none());
    assert!(current.provenance().source("source-00000000").is_none());
    let a = manager.prepare(
        current.revision(),
        DatasetUpdate::new(vec![DatasetChange::Put(record(count + 1, 3)?)]),
        Some(&mut provider),
    )?;
    let b = manager.prepare(
        current.revision(),
        DatasetUpdate::new(vec![DatasetChange::Put(record(count + 2, 3)?)]),
        Some(&mut provider),
    )?;
    manager.publish(a)?;
    assert!(matches!(manager.publish(b), Err(DatasetError::Conflict)));
    let start = Instant::now();
    for _ in 0..10000 {
        black_box(manager.snapshot()?);
    }
    let acquire_average_ns = start.elapsed().as_secs_f64() * 1e9 / 10000.0;
    println!(
        "{}",
        json!({"entries":count,"text_projection":positional,"initial_prepare_ms":initial_prepare_ms,"initial_publish_ms":initial_publish_ms,"changed_entries":changed,"changed_prepare_ms":changed_prepare_ms,"changed_publish_ms":changed_publish_ms,"metadata_prepare_ms":metadata_prepare_ms,"metadata_embedded_entries":0,"deletion_prepare_ms":deletion_prepare_ms,"acquire_average_ns":acquire_average_ns,"old_reader_preserved":true,"stale_publish_rejected":true,"method":"public synthetic source records and two-dimensional handwritten vectors; one observation per build/change phase; record generation outside preparation timing; snapshot acquisition average over 10000 iterations; retained old readers and competing prepared snapshots contribute to peak RSS; publication timings retain old views and exclude their later destruction"})
    );
    Ok(())
}
