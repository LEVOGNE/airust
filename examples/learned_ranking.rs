use airust::{
    learned_ranking::*,
    reranking::{rerank, RerankOptions},
    KnowledgeEntry,
};
use std::collections::BTreeSet;
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut examples = Vec::new();
    for i in 0..120 {
        let q = format!("topic{i} detail{i}");
        examples.push(TrainingExample::new(
            &q,
            format!("{q} explains the requested detail"),
            true,
        )?);
        examples.push(TrainingExample::new(
            &q,
            format!("unrelated{i} archive glossary"),
            false,
        )?);
    }
    let (ti, hi) = split_grouped(&examples, 25, 42)?;
    let training = ti.iter().map(|i| examples[*i].clone()).collect::<Vec<_>>();
    let holdout = hi.iter().map(|i| examples[*i].clone()).collect::<Vec<_>>();
    let tg = training.iter().map(|e| e.group()).collect::<BTreeSet<_>>();
    assert!(holdout.iter().all(|e| !tg.contains(e.group())));
    let mut extractor = LexicalFeatures;
    let start = Instant::now();
    let model = train(&training, &mut extractor, &TrainingOptions::default())?;
    let train_ms = start.elapsed().as_secs_f64() * 1000.;
    let evaluation = evaluate(&model, &holdout, &mut extractor)?;
    let restored = LinearModel::from_json(&model.to_json()?)?;
    assert_eq!(model.fingerprint()?, restored.fingerprint()?);
    let mut ranker = LinearReranker::new(&restored, &mut extractor)?;
    let docs = [
        KnowledgeEntry::new("a", "unrelated archive glossary"),
        KnowledgeEntry::new("b", "newtopic detail explains the requested detail"),
    ];
    let out = rerank(
        "newtopic detail",
        &docs,
        &mut ranker,
        &RerankOptions::new(2),
    )?;
    assert_eq!(out.results()[0].entry().id, "b");
    println!(
        "{}",
        serde_json::json!({"schema":1,"fixture":"authored lexical patterns, grouped query holdout, no semantic quality claim","train_examples":training.len(),"holdout":evaluation,"neutral_zero_baseline_correct":holdout.iter().filter(|e|e.relevant()).count(),"train_ms":train_ms,"training_report":model.report(),"schema_features":model.schema(),"weights":model.weights(),"model_fingerprint":model.fingerprint()?,"model_json_bytes":model.to_json()?.len(),"new_query_first":out.results()[0].entry().id})
    );
    Ok(())
}
