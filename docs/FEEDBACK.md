# Explicit local feedback

Enable `feedback`. It includes the existing reranking interface and SHA-256, without an inference model, database, server or telemetry collector.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use airust::feedback::*;
let mut ledger = FeedbackLedger::default();
let event = FeedbackEvent::new("vote-1", "workspace-a", "Rust ownership", "Ownership prevents memory errors", Rating::Helpful)?;
assert!(ledger.record(event.clone())?.changed);
assert!(!ledger.record(event)?.changed);
let evidence = ledger.evidence("workspace-a", "rust ownership", "Ownership prevents memory errors", 10)?;
assert_eq!(evidence.helpful, 1);
let restored = FeedbackLedger::from_json(&ledger.to_json()?, FeedbackLimits::default())?;
assert_eq!(restored.revision(), ledger.revision());
ledger.retract("vote-1")?;
# Ok(()) }
```

## Ledger identity and trust

FeedbackEvent::new(id, namespace, query, document, rating) accepts Helpful or Unhelpful. A stable event ID is an upsert key: identical replay changes nothing; replacement subtracts the old vote before adding the new one. retract removes it, and missing retractions are harmless. Changed operations increment a checked u64 revision; exhaustion fails before mutation. Receipt exposes changed/revision/events. events/event support inspection.

Query keys use core NFKC/lowercase/whitespace normalization, not stemming or semantic similarity. Documents use exact original bytes. A changed document is a different version; a different namespace is a different profile. The Reranker API receives text, not entry IDs: identical text in the same namespace/query shares feedback. Choose namespaces to separate applications, users, tenants or experiments. An event ID is not a verified user identity. The application must authenticate raters, enforce visibility, prevent duplicate voters with different IDs, and decide when a rating is trustworthy. Imported votes are untrusted unless the application verifies them.

Only SHA-256 fingerprints of query/document are retained. Namespace and event IDs remain plain text; choose opaque labels. Hashes are neither encryption nor anonymization and can be guessed with dictionary attacks. No data is uploaded. evidence returns counts plus at most min(max_ids,1000) contributing IDs with an explicit truncation flag. profile_fingerprint is deterministic over that namespace's sorted events, independent of unrelated namespaces and global revision. It identifies data, not authenticity.

## Explicit bounded ranking adjustment

FeedbackReranker::new(base, ledger, namespace, options) wraps a caller-owned Reranker. Existing rerank or RetrievalOptions can use it. The adjustment is:

`base_score + weight * (helpful - unhelpful) / (helpful + unhelpful + prior)`

Default weight=1, prior=2; validated weight is finite 0..100, prior 1..1,000,000. The bias magnitude is bounded by weight. It is not a calibrated probability; pick scale and evaluate against the base scorer's units. Zero weight disables the bias. This release remembers explicit query/document preferences; it does not generalize to unseen questions or train a model.

The recipe fingerprints base recipe, namespace profile and options. Base recipe changes before/after a provider call return RecipeChanged. Counts, finite scores and provider errors remain checked; errors do not produce partial diagnostics. last_adjustments describes the last successful batch (positions start at one), vote counts, document fingerprint, base score, boost and final score. It is separate from the response JSON budget and is not a complete multi-batch history. Existing rerank scope filtering happens before provider/feedback scoring. Direct score calls have no application authorization context.

## Limits, persistence and interruptions

FeedbackLimits defaults to 10,000 events and 8 MiB JSON. Supported maxima are 100,000 and 64 MiB; export/import check the selected cap. Import is schema-versioned and rejects unknown fields, duplicate IDs, invalid fingerprints/labels and inconsistent revision/count. Event labels are nonblank/control-free, up to 128 bytes. Query/document caps are 16 KiB/64 KiB; scoring caps a batch at 1,000 documents. Limit failures leave ledger operations unchanged. Export may fail when its byte cap is smaller than the in-memory ledger; choose limits together. Parsing is bounded by input bytes, not an exact RAM quota. Large record/import/export operations are synchronous; they do not claim preemptible I/O. Outer controlled reranking checkpoints between batches and before/after providers; arbitrary blocking providers still require their own timeouts.

Run feedback and feedback_benchmark. The authored preference fixture demonstrates baseline a -> rated b -> retracted a, not held-out quality. The 10,000-event benchmark verifies round-trip profile identity and records actual timings/resources. Evaluate trusted feedback on your own queries before enabling a bias. Existing knowledge records and sources are untouched. No DenkMaps/MCP integration is performed.
