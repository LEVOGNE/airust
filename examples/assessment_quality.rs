use airust::{retrieval::*, Engine, KnowledgeEntry};
use serde_json::json;
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cases: &[(&str, &[&str], bool)] = &[
        (
            "Rust ownership",
            &["Rust ownership prevents use-after-free."],
            true,
        ),
        (
            "SQLite atomic transactions",
            &["SQLite atomic transactions commit all changes together."],
            true,
        ),
        (
            "time budget",
            &["A time budget bounds cooperative work."],
            true,
        ),
        ("Häuser Türen", &["Häuser haben rote Türen."], true),
        (
            "immutable read views",
            &["Immutable read views preserve an earlier revision."],
            true,
        ),
        (
            "vehicle registration documents",
            &["Car registration requires identity papers."],
            true,
        ),
        ("Rust ownership", &["Rust scripts run locally."], false),
        (
            "SQLite transactions",
            &["SQLite installation guide."],
            false,
        ),
        (
            "Rust ownership",
            &["Rust language", "ownership glossary"],
            false,
        ),
        (
            "volcano eruption",
            &["Rust ownership prevents errors."],
            false,
        ),
        (
            "cancellation rollback",
            &["Cancellation support planned."],
            false,
        ),
        (
            "Rust ownership",
            &["Glossary headings: Rust, ownership. Explanations unavailable."],
            false,
        ),
    ];
    let mut rows = Vec::new();
    let mut baseline_false_accepts = 0;
    let mut false_accepts = 0;
    let mut false_rejects = 0;
    let started = Instant::now();
    for (query, texts, answerable) in cases {
        let index = Engine::from_entries(
            texts
                .iter()
                .enumerate()
                .map(|(i, t)| KnowledgeEntry::new(i.to_string(), *t)),
        )?;
        let result = Retriever::new(LexicalSource::new(&index)).query(
            query,
            &RetrievalOptions::new(3).with_evidence_policy(EvidencePolicy::default()),
        )?;
        let verdict = result.assessment().unwrap().verdict();
        let accepted = verdict == EvidenceVerdict::MeetsPolicy;
        baseline_false_accepts += usize::from(!answerable && !result.hits().is_empty());
        false_accepts += usize::from(!answerable && accepted);
        false_rejects += usize::from(*answerable && !accepted);
        rows.push(json!({"query":query,"texts":texts,"answerable_authored_label":answerable,"verdict":verdict,"delivered_hits":result.hits().len()}));
    }
    assert_eq!(false_accepts, 1);
    assert_eq!(false_rejects, 1);
    let index = Engine::from_entries([KnowledgeEntry::new(
        "a",
        "Rust ownership prevents memory errors",
    )])?;
    let mut retriever = Retriever::new(LexicalSource::new(&index));
    let mut timings = Vec::new();
    for assessed in [false, true] {
        let options = if assessed {
            RetrievalOptions::new(3).with_evidence_policy(EvidencePolicy::default())
        } else {
            RetrievalOptions::new(3)
        };
        let begin = Instant::now();
        let mut bytes = 0;
        for _ in 0..10000 {
            bytes += retriever.query("Rust ownership", &options)?.to_json().len();
        }
        timings.push(json!({"assessment":assessed,"queries":10000,"elapsed_ms":begin.elapsed().as_secs_f64()*1000.0,"serialized_bytes":bytes}));
    }
    println!(
        "{}",
        json!({"fixture":"authored-assessment-v1","case_count":cases.len(),"baseline_rule":"accept any delivered hit","baseline_false_accepts":baseline_false_accepts,"policy_false_accepts":false_accepts,"policy_false_rejects":false_rejects,"rows":rows,"timings":timings,"total_ms":started.elapsed().as_secs_f64()*1000.0,"limits":"Tiny authored lexical fixture; no independent semantic or answer quality benchmark."})
    );
    Ok(())
}
