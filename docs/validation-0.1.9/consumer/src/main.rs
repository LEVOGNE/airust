use airust::{compare, Context, Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::{error::Error, hint::black_box, time::Instant};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn snapshot(engine: &Engine, query: &str, algorithm: SearchAlgorithm) -> Value {
    serde_json::to_value(engine.search_with_options(
        query,
        &SearchOptions {
            algorithm,
            limit: 100,
            ..Default::default()
        },
    ))
    .unwrap()
}

fn load(connection: &Connection) -> Result<Engine> {
    let mut statement = connection.prepare("SELECT id, text, metadata FROM records ORDER BY id")?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut entries = Vec::new();
    for row in rows {
        let (id, text, metadata) = row?;
        entries.push(KnowledgeEntry::new(id, text).with_metadata(serde_json::from_str(&metadata)?));
    }
    Ok(Engine::from_entries(entries)?)
}

fn main() -> Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "scale") {
        return scale(args.get(1).ok_or("Missing entry count")?.parse()?);
    }
    let corpus = std::fs::read_to_string(
        args.first()
            .ok_or("Pass a corpus JSON file, or use scale ENTRY_COUNT")?,
    )?;
    let source: Vec<KnowledgeEntry> = serde_json::from_str(&corpus)?;
    let source_count = source.len();
    let started = Instant::now();
    let mut index = Engine::from_entries(source.clone())?;
    let build_ms = started.elapsed().as_secs_f64() * 1000.0;
    let query_terms: Vec<String> = source
        .iter()
        .flat_map(|entry| airust::text::tokenize(&entry.text))
        .filter(|word| word.chars().count() >= 4)
        .take(100)
        .collect();
    let started = Instant::now();
    for query in &query_terms {
        let hits = index.search(query, 10);
        assert!(!hits.is_empty());
        for hit in hits {
            assert!(airust::text::tokenize(&hit.entry.text).contains(query));
            assert!(hit.score.is_finite() && hit.score > 0.0);
        }
    }
    let query_ms = started.elapsed().as_secs_f64() * 1000.0;
    let restored = Engine::from_json(&index.to_json()?)?;
    assert_eq!(index.stats(), restored.stats());
    for query in &query_terms {
        for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            assert_eq!(
                snapshot(&index, query, algorithm),
                snapshot(&restored, query, algorithm)
            );
        }
    }
    let context = Context::new().with_metadata("collection", 0);
    for query in &query_terms {
        assert!(index
            .search_in_context(query, &context, 100)
            .iter()
            .all(|hit| context.includes(hit.entry)));
    }
    let mutations = source_count.min(200);
    for (n, entry) in source.iter().take(mutations).enumerate() {
        if n % 3 == 0 {
            index.remove(&entry.id);
        } else {
            index.upsert(
                KnowledgeEntry::new(&entry.id, format!("Geänderte Zulassung {n} Straße"))
                    .with_metadata(json!({"collection":n % 3})),
            )?;
        }
    }
    let rebuilt = Engine::from_entries(index.entries().cloned())?;
    assert_eq!(index.stats(), rebuilt.stats());
    assert_eq!(
        serde_json::to_value(index.duplicates())?,
        serde_json::to_value(rebuilt.duplicates())?
    );
    for query in query_terms
        .iter()
        .map(String::as_str)
        .chain(["geänderte", "straße", "zulassung"])
    {
        for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            assert_eq!(
                snapshot(&index, query, algorithm),
                snapshot(&rebuilt, query, algorithm)
            );
        }
    }
    for entry in index.entries().take(30) {
        assert_eq!(index.relate(&entry.id, 10)?, rebuilt.relate(&entry.id, 10)?);
        assert_eq!(
            serde_json::to_value(index.find_matches(&entry.text, 5))?,
            serde_json::to_value(rebuilt.find_matches(&entry.text, 5))?
        );
    }
    let mut connection = Connection::open_in_memory()?;
    connection.execute_batch(
        "CREATE TABLE records(id TEXT PRIMARY KEY, text TEXT NOT NULL, metadata TEXT NOT NULL)",
    )?;
    let mut mirror = Engine::new();
    for n in 0..300 {
        let id = format!("entry-{}", n % 41);
        let text = format!("Straße Zulassung Prüfung {}", n % 7);
        let metadata = json!({"collection":n % 4}).to_string();
        let transaction = connection.transaction()?;
        if n % 5 == 0 {
            transaction.execute("DELETE FROM records WHERE id=?1", [&id])?;
        } else {
            transaction.execute("INSERT INTO records VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET text=excluded.text,metadata=excluded.metadata", params![id,text,metadata])?;
        }
        if n % 11 == 0 {
            transaction.rollback()?;
        } else {
            transaction.commit()?;
            if n % 5 == 0 {
                mirror.remove(&id);
            } else {
                mirror.upsert(
                    KnowledgeEntry::new(&id, &text).with_metadata(serde_json::from_str(&metadata)?),
                )?;
            }
        }
        let rebuilt = load(&connection)?;
        assert_eq!(mirror.to_json()?, rebuilt.to_json()?);
        assert_eq!(mirror.stats(), rebuilt.stats());
        for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            assert_eq!(
                snapshot(&mirror, "Straße Zulassung", algorithm),
                snapshot(&rebuilt, "Straße Zulassung", algorithm)
            );
        }
    }
    let duplicate = compare(
        "  FÜHRERSCHEIN\tPRÜFEN ",
        "Fu\u{308}hrerschein pru\u{308}fen",
    );
    assert!(duplicate.exact);
    assert!(!compare("Straße", "Strasse").exact);
    assert!(!compare("Führerschein", "Fuehrerschein").exact);
    let typo = compare("Kfz-Zulassung durchführen", "Kfz-Zulassnug durchführen");
    assert!(typo.character >= 0.8);
    let semantic_a = "Ein Fahrzeug bei der Behörde anmelden";
    let semantic_b = "Kfz-Zulassung durchführen";
    let semantic = compare(semantic_a, semantic_b);
    assert_eq!(semantic.token, 0.0);
    let semantic_index = Engine::from_entries([
        KnowledgeEntry::new("a", semantic_a),
        KnowledgeEntry::new("b", semantic_b),
    ])?;
    assert!(semantic_index.relate("a", 10)?.is_empty());
    assert!(semantic_index
        .search("Kfz-Zulassung", 10)
        .iter()
        .all(|hit| hit.entry.id != "a"));
    let negation = Engine::from_entries([
        KnowledgeEntry::new("yes", "Fahrzeug bei der Behörde anmelden"),
        KnowledgeEntry::new("no", "Fahrzeug bei der Behörde nicht anmelden"),
    ])?;
    let proposal = negation.relate("yes", 1)?;
    assert_eq!(proposal[0].kind, airust::RelationKind::Related);
    assert!(proposal[0].score > 0.5);
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "airust_version":airust::VERSION, "source":"caller-supplied JSON corpus; no content included in report", "source_entries":source_count,
            "build_ms":build_ms, "warm_query_count":query_terms.len(), "warm_query_total_ms":query_ms,
            "real_corpus_mutations":mutations, "sqlite_in_memory_steps":300,
            "sqlite_version":rusqlite::version(), "rusqlite_dependency":"0.41.1",
            "checks":{"round_trip":true,"ranker_rebuild_parity":true,"context_filter":true,"duplicate_rebuild_parity":true,"fuzzy_rebuild_parity":true,"relation_rebuild_parity":true,"sqlite_commit_and_rollback_replay":true},
            "german":{"unicode_equivalence":duplicate,"typo":typo,"semantic_pair":semantic,"transliterations_are_not_exact":true,"synonym_relation_found":false,"opposing_statements_can_share_terms":true},
            "limits":"The test caller replays committed writes. AIRust does not observe SQLite automatically. No semantic quality, relevance labels, production sync or application performance claim."
        }))?
    );
    Ok(())
}

fn scale(count: usize) -> Result {
    if count == 0 {
        return Err("Entry count must be positive".into());
    }
    let started = Instant::now();
    let engine = Engine::from_entries((0..count).map(|n| {
        KnowledgeEntry::new(
            format!("id-{n:09}"),
            format!(
                "Wissen Straße Zulassung bucket{} Prüfung{}",
                n % 10000,
                n % 37
            ),
        )
    }))?;
    let build_ms = started.elapsed().as_secs_f64() * 1000.0;
    let mut results = Vec::new();
    for query in ["bucket7919", "Wissen Straße", "nichtvorhandenerbegriff"] {
        black_box(engine.search(query, 10));
        let mut samples = Vec::new();
        let mut hits = 0;
        for _ in 0..12 {
            let started = Instant::now();
            let found = black_box(engine.search(black_box(query), 10));
            samples.push(started.elapsed().as_secs_f64() * 1000.0);
            hits = found.len();
            assert!(found
                .iter()
                .all(|hit| hit.score.is_finite() && hit.score > 0.0));
        }
        samples.sort_by(f64::total_cmp);
        results.push(json!({"query":query,"hits":hits,"samples_ms":samples,"median_ms":(samples[5]+samples[6])/2.0,"max_ms":samples[11]}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"entries":count,"build_ms":build_ms,"queries":results,"limit":10,"samples_per_query":12,"corpus":"five tokens, German Unicode, rare buckets and common terms; synthetic; not semantic evaluation"})
        )?
    );
    Ok(())
}
