mod reference_eval;

use airust::{
    quality::{Judgments, UnjudgedPolicy},
    quality_gates::{CaseMeasurement, EvaluationRun},
};
use reference_eval::{Dataset, Indexes, Result};
use serde_json::{json, Value};
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 4 || args.len() > 7 {
        return Err("Usage: reference_evaluation DATASET SPLIT PROFILE NEW_OUTPUT [VECTORS|- [RERANKER_DIR RUNTIME_LIBRARY]]".into());
    }
    let started = Instant::now();
    let dataset = Dataset::load(Path::new(&args[0]), &args[1])?;
    let load_ms = started.elapsed().as_secs_f64() * 1000.0;
    let vectors = args
        .get(4)
        .filter(|path| path.as_str() != "-")
        .map(Path::new);
    let reranker = match (args.get(5), args.get(6)) {
        (Some(directory), Some(runtime)) => Some((Path::new(directory), Path::new(runtime))),
        (None, None) => None,
        _ => return Err("supply both reranker directory and runtime".into()),
    };
    let build_start = Instant::now();
    let mut indexes = Indexes::build(&dataset, &args[2], vectors, reranker)?;
    let build_ms = build_start.elapsed().as_secs_f64() * 1000.0;
    let dense_reference_max_difference = indexes.verify_dense_reference(
        &dataset
            .cases
            .iter()
            .map(|case| case.query.as_str())
            .collect::<Vec<_>>(),
    )?;
    let repetitions = dataset.policy["measurement"]["repetitions"]
        .as_u64()
        .ok_or("missing repetitions")? as usize;
    let mut output = Vec::<Value>::new();
    let mut measurements = Vec::new();
    let mut order: Vec<_> = (0..dataset.cases.len()).collect();
    let mut seed = dataset.policy["measurement"]["query_order_seed"]
        .as_u64()
        .ok_or("missing order seed")?;
    for upper in (1..order.len()).rev() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        order.swap(upper, (seed as usize) % (upper + 1));
    }
    for position in order {
        let case = &dataset.cases[position];
        let judgments = Judgments::new(
            dataset
                .judgments
                .iter()
                .filter(|row| row.query_id == case.id)
                .map(|row| (row.document_id.clone(), row.grade)),
        )?;
        let quality = indexes.query(&case.query, 10)?;
        dataset.validate_response(
            &quality,
            dataset.policy["corpus"]["collection"]
                .as_str()
                .ok_or("collection")?,
        )?;
        let ids: Vec<_> = quality.hits().iter().map(|hit| hit.id.clone()).collect();
        let mut times = Vec::new();
        let mut final_reply = None;
        for iteration in 0..=repetitions {
            let start = Instant::now();
            let response = indexes.query(&case.query, 6)?;
            let encoded = serde_json::to_string(&response)?;
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            dataset.validate_response(
                &response,
                dataset.policy["corpus"]["collection"]
                    .as_str()
                    .ok_or("collection")?,
            )?;
            if response.hits().len() > 6
                || encoded.len() > 65536
                || response
                    .hits()
                    .iter()
                    .map(|hit| hit.excerpt.len())
                    .sum::<usize>()
                    > 12000
            {
                return Err("delivered response exceeds the frozen budget".into());
            }
            if iteration > 0 {
                times.push(elapsed_ms);
            }
            final_reply = Some(response);
        }
        let response = final_reply.ok_or("missing measured response")?;
        let delivered_hit = case.reference_quote.as_ref().is_some_and(|quote| {
            response
                .hits()
                .iter()
                .any(|hit| hit.excerpt.contains(quote))
        });
        let mean = times.iter().sum::<f64>() / times.len() as f64;
        let measurement = CaseMeasurement::evaluate(
            &case.id,
            &case.query,
            &ids,
            &judgments,
            10,
            UnjudgedPolicy::NonRelevant,
            Some(Duration::from_secs_f64(mean / 1000.0)),
        )?;
        let metrics =
            airust::quality::evaluate_ranking(&ids, &judgments, 10, UnjudgedPolicy::NonRelevant)?;
        if case.answerable {
            measurements.push(measurement);
        }
        output.push(json!({"id": case.id, "family": case.family, "language": case.language, "answerable": case.answerable, "ranked_ids": ids, "quality_evidence": quality.hits().iter().map(|hit| json!({"id": hit.id, "score_bits": hit.evidence.score.to_bits(), "terms": hit.evidence.matched_terms, "rerank_score": hit.rerank_score})).collect::<Vec<_>>(), "delivered_ids": response.hits().iter().map(|hit| &hit.id).collect::<Vec<_>>(), "reference_hit_at_6": delivered_hit, "candidate_returned_on_negative": !case.answerable && !response.hits().is_empty(), "metrics": metrics, "latencies_ms": times, "response_bytes": response.to_json().len(), "text_bytes": response.hits().iter().map(|hit| hit.excerpt.len()).sum::<usize>(), "recipe": response.recipe()}));
    }
    output.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let run = EvaluationRun::new(
        dataset.manifest["dataset"].as_str().ok_or("dataset name")?,
        dataset.manifest["artifacts"]["corpus.jsonl"]["sha256"]
            .as_str()
            .ok_or("corpus identity")?,
        &args[2],
        measurements,
        None,
    )?;
    let result = json!({"format_version": 1, "airust_version": airust::VERSION, "profile": args[2], "split": args[1], "dataset_identity": dataset.manifest["identity_sha256"], "corpus_entries": dataset.rows.len(), "data_load_ms": load_ms, "build_ms": build_ms, "sqlite_version": rusqlite::version(), "sqlite_token_vocabulary_and_frequencies_checked": args[2].starts_with("sqlite"), "dense_reference_max_difference": dense_reference_max_difference, "timing_scope": "Six-hit RetrievalResponse including complete JSON serialization; index construction and externally prepared query embeddings excluded. Quality at ten is an untimed internal probe.", "scope_leaks": 0, "span_mismatches": 0, "cases": output, "quality_run": run});
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?
        .write_all(serde_json::to_string(&result)?.as_bytes())?;
    println!(
        "{}",
        json!({"profile": args[2], "split": args[1], "cases": dataset.cases.len(), "build_ms": build_ms})
    );
    Ok(())
}
