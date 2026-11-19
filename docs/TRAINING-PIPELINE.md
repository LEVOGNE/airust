# Controlled training pipeline

Enable `training-pipeline`. This connects explicit local feedback, existing trainable ranking and quality gates. It collects no ratings, trains no model and changes no active model automatically.

## Curate trusted, current examples

TrainingItem::new(id, query, document) supplies an already authorized current text pair. with_group can keep paraphrases/near duplicates together. CurationPolicy::new(trusted_event_ids) requires an explicit trusted set; an empty set selects no ratings. IDs are global within a ledger. The application must authenticate raters, enforce access and deduplicate voters before granting trust. Different event IDs are not proof of different people.

curate_feedback(namespace, ledger, items, policy) matches the exact feedback query/document fingerprints in that namespace. Unjudged items are omitted, never turned into negative labels. Defaults require two votes and a nonzero majority; with_minimums changes those thresholds. Ties and insufficient margins are omitted. Duplicate content is represented once, choosing the lexicographically first item ID; inconsistent duplicate groups and repeated item IDs are errors. A helpful/unhelpful majority is a chosen training label, not established ground truth.

TrainingDataset retains reviewed texts, groups, counts and contributing trusted event IDs. examples() exposes validated TrainingExample values. fingerprint binds the dataset; is_current checks the namespace's feedback profile. Any namespace change conservatively requests review, including unrelated/untrusted events. Source-text freshness remains caller-owned: supply a consistent authorized source snapshot and regenerate items when its content or access changes.

TrainingDataset::to_json/from_json provides schema-versioned, checksummed artifacts. Export includes raw query/document text and plain event/item IDs. Store it privately; it is neither anonymized nor uploaded automatically. Import checks counts, labels, groups, unique pairs/IDs, vote provenance, limits and checksum. A checksum is not authentication; verify artifacts from other trust boundaries. The library cannot reconstruct raters or prove that imported labels were genuinely reviewed.

Inputs cap at 10,000 items, 64 MiB paired text and the existing 16 KiB/64 KiB query/document limits. Dataset JSON caps at 16 MiB. Trusted event sets cap at 100,000 IDs. Limits constrain this format, not exact RAM. These curation/import steps are synchronous.

## Train with separate query groups

train_grouped(dataset, extractor, options, holdout_percent, seed) uses existing split_grouped, trains only training indices and evaluates only held-out indices. GroupedTraining retains dataset identity, split indices, model and Evaluation. Use explicit shared groups for paraphrases; normalization alone cannot detect all leakage. TrainingOptions controls bounded optimization and optional cooperative execution. Held-out classification loss is not a retrieval-ranking quality guarantee. Evaluate the candidate's real rankings separately using quality-gates before promotion.

## Explicit model promotion and rollback

ModelRegistry::new(identity, capacity) creates a caller-owned in-memory slot with 2..32 retained models. current returns an immutable model; clone it when readers must retain a previous model. revision includes registry identity, generation and active fingerprint. promote(expected, model, baseline_run, candidate_run, gate_report) requires an exact revision, a Passed gate for those exact runs, the candidate's actual model recipe, and the current model's baseline recipe when a model is active. Failed/undetermined/mismatched checks leave the slot unchanged.

rollback(expected) restores a retained predecessor. Generations advance, so A -> B -> A does not accept an old A revision. Identical re-promotion is an idempotent no-op. Missing history or exhausted generations are errors. Registry JSON preserves identity, generation and bounded validated model history; authenticate persisted state and use a unique caller-owned identity. This is not cross-process locking, automatic deployment or a database transaction. The application coordinates persistence and concurrency.

The registry verifies evaluation/report identity and explicit thresholds. It cannot establish that judgments are correct, that evaluation was held out, or that hardware/resource measurements are honest. Preserve dataset/model/run identities and review records externally. No DenkMaps/MCP integration is performed.

## Complete curation example

```rust
use airust::{feedback::*, learned_ranking::*, training_pipeline::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = FeedbackLedger::default();
    for (id, doc, rating) in [
        ("yes", "q answer", Rating::Helpful),
        ("no", "unrelated", Rating::Unhelpful),
    ] {
        ledger.record(FeedbackEvent::new(id, "demo", "q", doc, rating)?)?;
    }
    let policy = CurationPolicy::new(["yes".into(), "no".into()])?.with_minimums(1, 1)?;
    let items = [
        TrainingItem::new("answer", "q", "q answer")?,
        TrainingItem::new("other", "q", "unrelated")?,
    ];
    let data = curate_feedback("demo", &ledger, &items, &policy)?;
    let restored = TrainingDataset::from_json(&data.to_json()?)?;
    assert_eq!(restored.fingerprint()?, data.fingerprint()?);
    let model = train(
        data.examples(),
        &mut LexicalFeatures,
        &TrainingOptions::default(),
    )?;
    println!("{} examples; dataset {}; model {}",data.len(),data.fingerprint()?,model.fingerprint()?);
    Ok(())
}

```
