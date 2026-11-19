use airust::{
    quality::*, quality_gates::*, Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader},
    path::Path,
    time::Instant,
};
#[derive(Deserialize)]
struct Row {
    #[serde(rename = "_id")]
    id: String,
    text: String,
    #[serde(default)]
    title: String,
}
fn rows(path: &Path) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
    if fs::metadata(path)?.len() > 512 * 1024 * 1024 {
        return Err("input file limit".into());
    }
    let mut rows = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for line in BufReader::new(fs::File::open(path)?).lines() {
        let line = line?;
        if line.len() > 1024 * 1024 || rows.len() >= 1000000 {
            return Err("row limit".into());
        }
        let row: Row = serde_json::from_str(&line)?;
        if row.id.trim().is_empty() || row.id.len() > 4096 || !ids.insert(row.id.clone()) {
            return Err("empty/oversized/duplicate row ID".into());
        }
        rows.push(row);
    }
    Ok(rows)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = std::env::args()
        .nth(1)
        .ok_or("usage: beir_evaluation DATASET_DIR")?;
    let dir = Path::new(&arg);
    let corpus = dir.join("corpus.jsonl");
    if fs::metadata(&corpus)?.len() > 512 * 1024 * 1024 {
        return Err("corpus byte limit".into());
    }
    let bytes = fs::read(&corpus)?;
    if bytes.len() > 512 * 1024 * 1024 {
        return Err("corpus byte limit".into());
    }
    let fingerprint = format!("{:x}", Sha256::digest(&bytes));
    drop(bytes);
    let index = Engine::from_entries(
        rows(&corpus)?
            .into_iter()
            .map(|r| KnowledgeEntry::new(r.id, format!("{} {}", r.title, r.text))),
    )?;
    let queries = rows(&dir.join("queries.jsonl"))?
        .into_iter()
        .map(|r| (r.id, r.text))
        .collect::<BTreeMap<_, _>>();
    let mut judgments = BTreeMap::<String, Vec<(String, u8)>>::new();
    let qrels = dir.join("qrels/test.tsv");
    if fs::metadata(&qrels)?.len() > 64 * 1024 * 1024 {
        return Err("qrels byte limit".into());
    }
    for line in BufReader::new(fs::File::open(qrels)?).lines().skip(1) {
        let line = line?;
        let p = line.split('\t').collect::<Vec<_>>();
        if p.len() != 3 {
            return Err("invalid qrels".into());
        }
        judgments
            .entry(p[0].into())
            .or_default()
            .push((p[1].into(), p[2].parse()?));
    }
    if judgments.len() > 10000 {
        return Err("query limit".into());
    }
    let mut runs = Vec::new();
    for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
        let mut cases = Vec::new();
        for (id, grades) in &judgments {
            let query = queries.get(id).ok_or("missing query")?;
            let labels = Judgments::new(grades.clone())?;
            let start = Instant::now();
            let ranking = index.search_with_options(
                query,
                &SearchOptions {
                    limit: 10,
                    algorithm,
                    ..SearchOptions::default()
                },
            );
            let elapsed = start.elapsed();
            let ids = ranking
                .iter()
                .map(|r| r.entry.id.as_str())
                .collect::<Vec<_>>();
            cases.push(CaseMeasurement::evaluate(
                id,
                query,
                &ids,
                &labels,
                10,
                UnjudgedPolicy::NonRelevant,
                Some(elapsed),
            )?);
        }
        runs.push(EvaluationRun::new(
            "external-beir-test",
            &fingerprint,
            format!("{algorithm:?}"),
            cases,
            None,
        )?);
    }
    let report = compare_runs(&runs[0], &runs[1], &GatePolicy::default())?;
    println!(
        "{}",
        serde_json::json!({"schema_version":1,"scope":"external retrieval labels; not medical guidance or application validation","corpus_entries":index.len(),"baseline":runs[0],"candidate":runs[1],"comparison":report})
    );
    Ok(())
}
