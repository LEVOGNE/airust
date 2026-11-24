use airust::documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace};
use airust::execution::{CancellationToken, WorkControl};
use airust::retrieval::{
    select_excerpt, ExcerptAnalyzer, ExcerptOptions, RankingEvidence, RankingKind,
};
use airust::text_search::{AnalyzerConfig, TermOperator, TextIndex, TextSearchOptions};
use airust::{
    text, Context, Engine, IndexStats, KnowledgeEntry, MatchOptions, MatchStrategy,
    SearchAlgorithm, SearchOptions,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

pub struct LexicalCampaign;

impl LexicalCampaign {
    pub fn exercise(data: &[u8]) {
        let mut input = Input::new(data);
        ParsedCorpus::run(&mut input);
        SparseScore::run();
        TokenTail::run();
        RepeatedBoundary::run();
        if data.len() >= 2 && data[0] == 0xA5 && data[1] == 0x5A {
            WideCursor::run();
        }
        let sample = input.take(256);
        let _ = Engine::from_json(&String::from_utf8_lossy(sample));
    }
}

struct Input<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Input<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, at: 0 }
    }

    fn byte(&mut self) -> u8 {
        if self.at < self.data.len() {
            let byte = self.data[self.at];
            self.at += 1;
            byte
        } else {
            0
        }
    }

    fn take(&mut self, max: usize) -> &'a [u8] {
        let available = self.data.len().saturating_sub(self.at);
        let size = max.min(available);
        let slice = &self.data[self.at..self.at + size];
        self.at += size;
        slice
    }
}

struct ParsedCorpus;

impl ParsedCorpus {
    fn run(input: &mut Input<'_>) {
        let count = (input.byte() as usize) % 7;
        let mut engine = Engine::new();
        for index in 0..count {
            let text = Self::text(input);
            let mut entry = KnowledgeEntry::new(format!("d{index}"), text);
            if input.byte().is_multiple_of(2) {
                entry = entry.with_metadata(serde_json::json!({"k": input.byte() % 3}));
            }
            let _ = engine.upsert(entry);
        }
        let _ = engine.upsert(KnowledgeEntry::new("", "x"));
        let _ = engine.upsert(KnowledgeEntry::new("blank", " \t\n"));
        let first = engine.entries().next().cloned();
        if let Some(mut changed) = first {
            changed.text.push_str(" extra");
            let _ = engine.upsert(changed);
        }
        Formula::assert_index(&engine, input.byte());
        NeighborChecks::run(&engine, input.byte());
        if input.byte().is_multiple_of(3) {
            CacheProbe::run(&engine, input.byte());
        }
    }

    fn text(input: &mut Input<'_>) -> String {
        let kind = input.byte();
        let size = (input.byte() as usize) % 37;
        let raw = input.take(size);
        match kind % 5 {
            0 => String::from_utf8_lossy(raw).into_owned(),
            1 => {
                const PIECES: [&str; 14] = [
                    "a",
                    "b",
                    "z",
                    "A",
                    " ",
                    "\n",
                    "\t",
                    "0",
                    "é",
                    "ß",
                    "中",
                    "-",
                    "'",
                    "a\u{0301}",
                ];
                let mut text = String::new();
                for byte in raw {
                    text.push_str(PIECES[(*byte as usize) % PIECES.len()]);
                }
                text
            }
            2 => {
                let word = Self::word(raw);
                let copies = 1 + (kind as usize % 5);
                std::iter::repeat_n(word, copies)
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            3 => format!("qa\u{000b}qa {}", String::from_utf8_lossy(raw)),
            _ => raw
                .iter()
                .map(|byte| if byte.is_multiple_of(2) { 'A' } else { 'b' })
                .collect(),
        }
    }

    fn word(raw: &[u8]) -> String {
        let mut word = String::new();
        for byte in raw {
            if byte.is_ascii_alphanumeric() {
                word.push(byte.to_ascii_lowercase() as char);
            }
        }
        if word.is_empty() {
            "qa".to_owned()
        } else {
            word
        }
    }
}

struct ModelDoc {
    id: String,
    terms: BTreeMap<String, usize>,
    length: usize,
    metadata: Option<serde_json::Value>,
}

struct PreparedTerm {
    term: String,
    frequency: f64,
    idf: f64,
}

struct Hit {
    id: String,
    score: f64,
    terms: Vec<String>,
}

struct Formula;

impl Formula {
    fn assert_index(engine: &Engine, salt: u8) {
        let entries: Vec<KnowledgeEntry> = engine.entries().cloned().collect();
        let fresh =
            Engine::from_entries(entries.iter().cloned()).expect("accepted entries rebuild");
        assert_eq!(fresh.stats(), engine.stats());
        let json = engine.to_json().expect("index json");
        let restored = Engine::from_json(&json).expect("index json restores");
        assert_eq!(restored.stats(), engine.stats());
        let model = Self::model(&entries);
        assert_eq!(engine.stats(), Self::stats(&model));
        let configs = [
            airust::Bm25Config::default(),
            airust::Bm25Config::new(0.4, 0.0).expect("k1 below one"),
            airust::Bm25Config::new(1.0, 0.25).expect("k1 boundary"),
            airust::Bm25Config::new(3.5, 1.0).expect("k1 above one"),
        ];
        let limit = (salt as usize) % 5;
        let queries = Self::queries(salt);
        for query in &queries {
            for config in configs {
                for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
                    let options = SearchOptions {
                        limit,
                        algorithm,
                        bm25: config,
                        context: Self::context(salt, &model),
                    };
                    Self::assert_search(engine, &fresh, &model, query, &options);
                }
            }
        }
        let mut twin = engine.clone();
        let before = Self::signature(&engine.search("qa", 4));
        if let Some(entry) = entries.first() {
            twin.remove(&entry.id);
            let mut replacement = entry.clone();
            replacement.text = format!("{} replacement", entry.text);
            let _ = twin.upsert(replacement);
        }
        assert_eq!(before, Self::signature(&engine.search("qa", 4)));
        let _ = twin.duplicates();
    }

    fn queries(salt: u8) -> Vec<String> {
        let base = match salt % 4 {
            0 => "qa rust".to_owned(),
            1 => "QA".to_owned(),
            2 => "café cafe\u{301}".to_owned(),
            _ => "t00 missing".to_owned(),
        };
        let lower = base.to_ascii_lowercase();
        let vertical = base.replace(' ', "\u{000b}");
        let nbsp = base.replace(' ', "\u{00a0}");
        vec![base, lower, vertical, nbsp, format!("qa\u{000b}qa {salt}")]
    }

    fn context(salt: u8, model: &[ModelDoc]) -> Option<Context> {
        match salt % 5 {
            0 => None,
            1 => model
                .first()
                .map(|doc| Context::new().with_entry(doc.id.clone())),
            2 => Some(Context::new().with_metadata("k", 1)),
            3 => Some(Context::new().with_entry("missing")),
            _ => Some(Context::new()),
        }
    }

    fn assert_search(
        engine: &Engine,
        fresh: &Engine,
        model: &[ModelDoc],
        query: &str,
        options: &SearchOptions,
    ) {
        let actual = engine.search_with_options(query, options);
        let rebuilt = fresh.search_with_options(query, options);
        let expected = Self::hits(model, query, options);
        assert_eq!(
            Self::signature(&actual),
            expected,
            "query {query:?} limit {} k1 {} b {} alg {:?}",
            options.limit,
            options.bm25.k1(),
            options.bm25.b(),
            options.algorithm
        );
        assert_eq!(Self::signature(&actual), Self::signature(&rebuilt));
    }

    fn signature(results: &[airust::SearchResult<'_>]) -> Vec<(String, u64, Vec<String>)> {
        results
            .iter()
            .map(|result| {
                (
                    result.entry.id.clone(),
                    result.score.to_bits(),
                    result.matched_terms.clone(),
                )
            })
            .collect()
    }

    fn model(entries: &[KnowledgeEntry]) -> Vec<ModelDoc> {
        entries
            .iter()
            .map(|entry| {
                let terms = text::term_frequencies(&entry.text);
                let length = terms.values().sum();
                ModelDoc {
                    id: entry.id.clone(),
                    terms,
                    length,
                    metadata: entry.metadata.clone(),
                }
            })
            .collect()
    }

    fn stats(model: &[ModelDoc]) -> IndexStats {
        let mut terms = BTreeSet::new();
        let mut tokens = 0usize;
        for doc in model {
            tokens += doc.length;
            terms.extend(doc.terms.keys().cloned());
        }
        IndexStats {
            entries: model.len(),
            terms: terms.len(),
            tokens,
        }
    }

    fn hits(
        model: &[ModelDoc],
        query: &str,
        options: &SearchOptions,
    ) -> Vec<(String, u64, Vec<String>)> {
        if options.limit == 0 || model.is_empty() {
            return Vec::new();
        }
        let count = model.len();
        let mut document_frequency: BTreeMap<String, usize> = BTreeMap::new();
        let mut tokens = 0usize;
        for doc in model {
            tokens += doc.length;
            for term in doc.terms.keys() {
                *document_frequency.entry(term.clone()).or_default() += 1;
            }
        }
        let average = tokens as f64 / count as f64;
        let frequencies = text::term_frequencies(query);
        let mut prepared = Vec::new();
        for (term, frequency) in frequencies {
            if let Some(df) = document_frequency.get(&term).copied() {
                let idf = match options.algorithm {
                    SearchAlgorithm::Bm25 => Self::bm25_idf(count, df),
                    SearchAlgorithm::Tfidf => Self::tfidf_idf(count, df),
                };
                prepared.push(PreparedTerm {
                    term,
                    frequency: frequency as f64,
                    idf,
                });
            }
        }
        let query_norm = prepared
            .iter()
            .map(|term| (term.frequency * term.idf).powi(2))
            .sum::<f64>()
            .sqrt();
        let mut hits = Vec::new();
        for doc in model {
            if !Self::allows(options.context.as_ref(), doc) {
                continue;
            }
            let order = Self::contribution_order(&prepared, &doc.terms);
            let score = match options.algorithm {
                SearchAlgorithm::Bm25 => {
                    let norm = Self::length_norm(options.bm25.b(), doc.length, average);
                    let mut score = 0.0;
                    for index in &order {
                        let term = &prepared[*index];
                        let tf = doc.terms[&term.term];
                        score += Self::term_score(options.bm25.k1(), tf, term.idf, norm);
                    }
                    score
                }
                SearchAlgorithm::Tfidf => {
                    let mut dot = 0.0;
                    for index in &order {
                        let term = &prepared[*index];
                        let tf = doc.terms[&term.term] as f64;
                        dot += term.frequency * tf * term.idf * term.idf;
                    }
                    let mut sum = 0.0;
                    for (term, tf) in &doc.terms {
                        let idf = Self::tfidf_idf(count, document_frequency[term]);
                        sum += (*tf as f64 * idf).powi(2);
                    }
                    let norm = sum.sqrt();
                    if norm > 0.0 && query_norm > 0.0 {
                        (dot / (norm * query_norm)).clamp(0.0, 1.0)
                    } else {
                        0.0
                    }
                }
            };
            if score > 0.0 && score.is_finite() {
                hits.push(Hit {
                    id: doc.id.clone(),
                    score,
                    terms: order
                        .into_iter()
                        .map(|index| prepared[index].term.clone())
                        .collect(),
                });
            }
        }
        hits.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.id.cmp(&right.id))
        });
        hits.truncate(options.limit);
        hits.into_iter()
            .map(|hit| (hit.id, hit.score.to_bits(), hit.terms))
            .collect()
    }

    fn contribution_order(
        prepared: &[PreparedTerm],
        terms: &BTreeMap<String, usize>,
    ) -> Vec<usize> {
        let mut order = Vec::new();
        if prepared.len() > 32 && prepared.len() > terms.len().saturating_mul(2) {
            for term in terms.keys() {
                if let Ok(index) = prepared.binary_search_by(|item| item.term.cmp(term)) {
                    order.push(index);
                }
            }
        } else {
            for (index, term) in prepared.iter().enumerate() {
                if terms.contains_key(&term.term) {
                    order.push(index);
                }
            }
        }
        order
    }

    fn allows(context: Option<&Context>, doc: &ModelDoc) -> bool {
        let Some(context) = context else {
            return true;
        };
        if !context.entry_ids.is_empty() && !context.entry_ids.contains(&doc.id) {
            return false;
        }
        context.metadata.iter().all(|(key, value)| {
            doc.metadata
                .as_ref()
                .and_then(serde_json::Value::as_object)
                .and_then(|object| object.get(key))
                == Some(value)
        })
    }

    fn length_norm(b: f64, length: usize, average: f64) -> f64 {
        1.0 - b + b * length as f64 / average
    }

    fn term_score(k1: f64, tf: usize, idf: f64, norm: f64) -> f64 {
        let tf = tf as f64;
        if k1 >= 1.0 {
            idf * tf * (1.0 + 1.0 / k1) / (norm + tf / k1)
        } else {
            idf * tf * (k1 + 1.0) / (tf + k1 * norm)
        }
    }

    fn bm25_idf(count: usize, df: usize) -> f64 {
        (1.0 + (count as f64 - df as f64 + 0.5) / (df as f64 + 0.5)).ln()
    }

    fn tfidf_idf(count: usize, df: usize) -> f64 {
        ((count as f64 + 1.0) / (df as f64 + 1.0)).ln() + 1.0
    }
}

struct SparseScore;

impl SparseScore {
    fn run() {
        let mut engine = Engine::new();
        let _ = engine.upsert(KnowledgeEntry::new("s0", "t00"));
        let _ = engine.upsert(KnowledgeEntry::new("s1", "t01 t02"));
        let _ = engine.upsert(KnowledgeEntry::new("s2", "t03"));
        let _ = engine.upsert(KnowledgeEntry::new("s3", "t00 t47"));
        let query = (0..48)
            .map(|index| format!("t{index:02}"))
            .collect::<Vec<_>>()
            .join(" ");
        let options = SearchOptions {
            limit: 4,
            ..SearchOptions::default()
        };
        let model = Formula::model(&engine.entries().cloned().collect::<Vec<_>>());
        Formula::assert_search(&engine, &engine, &model, &query, &options);
        let tfidf = SearchOptions {
            algorithm: SearchAlgorithm::Tfidf,
            ..options
        };
        Formula::assert_search(&engine, &engine, &model, &query, &tfidf);
    }
}

struct TokenTail;

impl TokenTail {
    fn run() {
        let mut engine = Engine::new();
        let _ = engine.upsert(KnowledgeEntry::new("tail", "a ".repeat(1100)));
        let options = SearchOptions {
            limit: 1,
            algorithm: SearchAlgorithm::Tfidf,
            ..SearchOptions::default()
        };
        let model = Formula::model(&engine.entries().cloned().collect::<Vec<_>>());
        Formula::assert_search(&engine, &engine, &model, "a", &options);
    }
}

struct RepeatedBoundary;

impl RepeatedBoundary {
    fn run() {
        let mut engine = Engine::new();
        let _ = engine.upsert(KnowledgeEntry::new("edge", "a"));
        let model = Formula::model(&engine.entries().cloned().collect::<Vec<_>>());
        let options = SearchOptions {
            limit: 1,
            ..SearchOptions::default()
        };
        for query in ["a ".repeat(2048), format!("{}a", "a ".repeat(2048))] {
            Formula::assert_search(&engine, &engine, &model, &query, &options);
        }
    }
}

struct WideCursor;

impl WideCursor {
    fn run() {
        let mut engine = Engine::new();
        for index in 0..1100 {
            let _ = engine.upsert(KnowledgeEntry::new(format!("w{index:04}"), "rust"));
        }
        let options = SearchOptions {
            limit: 5,
            ..SearchOptions::default()
        };
        let entries: Vec<_> = engine.entries().cloned().collect();
        let fresh = Engine::from_entries(entries.iter().cloned()).expect("wide rebuild");
        let model = Formula::model(&entries);
        Formula::assert_search(&engine, &fresh, &model, "rust", &options);
    }
}

struct NeighborChecks;

impl NeighborChecks {
    fn run(engine: &Engine, salt: u8) {
        let _ = engine.duplicates();
        let _ = engine.find_matches("qa", (salt as usize) % 4);
        for strategy in [
            MatchStrategy::Exact,
            MatchStrategy::Fuzzy,
            MatchStrategy::TokenOverlap,
        ] {
            let options = MatchOptions {
                strategy,
                min_similarity: [0.0, 0.8, 1.0][(salt as usize) % 3],
                limit: 3,
            };
            let matched = engine
                .match_with_options("qa rust", &options)
                .expect("similarity is in range");
            let again = engine
                .match_with_options("qa rust", &options)
                .expect("similarity is in range");
            assert_eq!(
                matched
                    .iter()
                    .map(|item| (item.entry.id.clone(), item.score.to_bits()))
                    .collect::<Vec<_>>(),
                again
                    .iter()
                    .map(|item| (item.entry.id.clone(), item.score.to_bits()))
                    .collect::<Vec<_>>()
            );
        }
        if let Some(entry) = engine.entries().next() {
            let _ = engine.relate(&entry.id, 3);
            let missing = engine.relate("missing", 3);
            assert!(missing.is_err());
        }
        let query = "qa";
        let options = SearchOptions {
            limit: 3,
            ..SearchOptions::default()
        };
        let control = WorkControl::new(CancellationToken::new());
        let controlled = engine
            .search_controlled(query, &options, &control)
            .expect("open control");
        assert_eq!(
            Formula::signature(&controlled),
            Formula::signature(&engine.search_with_options(query, &options))
        );
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        let stopped = engine.search_controlled(query, &options, &WorkControl::new(cancelled));
        assert!(stopped.is_err());
        let _ = WorkControl::new(CancellationToken::new()).with_timeout(Duration::from_millis(1));
        Self::text(engine);
        Self::excerpt(salt);
        Self::document(salt);
    }

    fn text(engine: &Engine) {
        for config in [
            AnalyzerConfig::default()
                .with_limits(4096, 8192, 256)
                .expect("text limits"),
            AnalyzerConfig::german()
                .with_limits(4096, 8192, 256)
                .expect("german limits"),
        ] {
            let mut index = TextIndex::new(config);
            for entry in engine.entries().cloned() {
                let _ = index.upsert(entry);
            }
            let mut options = TextSearchOptions::new(3).with_operator(TermOperator::Any);
            if let Ok(limited) = options.clone().with_query_limits(256, 32, 4, 8) {
                options = limited;
            }
            for operator in [TermOperator::Any, TermOperator::All] {
                let search = options.clone().with_operator(operator);
                if let Ok(results) = index.search("qa Häuser", &search) {
                    for result in results {
                        assert!(result.score.is_finite());
                        for phrase in &result.phrases {
                            for span in &phrase.spans {
                                assert!(span.start <= span.end);
                                assert!(result.entry.text.get(span.start..span.end).is_some());
                            }
                        }
                    }
                }
            }
            let before = index.search("qa", &TextSearchOptions::new(2));
            let mut twin = index.clone();
            if let Some(entry) = index.entries().next().cloned() {
                twin.remove(&entry.id);
            }
            let after = index.search("qa", &TextSearchOptions::new(2));
            match (before, after) {
                (Ok(left), Ok(right)) => assert_eq!(
                    left.iter()
                        .map(|item| (item.entry.id.clone(), item.score.to_bits()))
                        .collect::<Vec<_>>(),
                    right
                        .iter()
                        .map(|item| (item.entry.id.clone(), item.score.to_bits()))
                        .collect::<Vec<_>>()
                ),
                (Err(_), Err(_)) => {}
                _ => panic!("clone changed text search outcome"),
            }
        }
    }

    fn excerpt(salt: u8) {
        let text = "alpha beta café. alpha";
        let query = if salt.is_multiple_of(2) { "alpha" } else { " " };
        let mut evidence = RankingEvidence::new(RankingKind::Bm25, 1.0);
        evidence.matched_terms = vec!["alpha".to_owned()];
        for options in [
            ExcerptOptions::default(),
            ExcerptOptions::default().with_paragraph_boundaries(),
            ExcerptOptions::default().with_leading_context(),
        ] {
            for budget in [0usize, 1, 8, 24] {
                if let Ok(selected) = select_excerpt(
                    text,
                    query,
                    &evidence,
                    &ExcerptAnalyzer::Legacy,
                    &options,
                    budget,
                ) {
                    assert_eq!(
                        Some(selected.text),
                        text.get(selected.span.start..selected.span.end)
                    );
                }
            }
        }
    }

    fn document(salt: u8) {
        let text = if salt.is_multiple_of(2) {
            "alpha beta".to_owned()
        } else {
            "café\nbeta".to_owned()
        };
        let limits = DocumentLimits {
            max_source_bytes: 4096,
            max_text_bytes: 4096,
            max_pages: 4,
            max_id_bytes: 32,
            max_recipe_bytes: 32,
            max_json_bytes: 16384,
        };
        let Ok(source) = SourceFingerprint::from_bytes(text.as_bytes(), &limits) else {
            return;
        };
        let Ok(snapshot) =
            DocumentSnapshot::new("doc", source, "fuzz", TextSpace::Supplied, text, &limits)
        else {
            return;
        };
        let json = snapshot
            .to_json(limits.max_json_bytes)
            .expect("snapshot json");
        let restored = DocumentSnapshot::from_json(&json, &limits).expect("snapshot restores");
        assert_eq!(snapshot.text(), restored.text());
        let mut damaged = json.into_bytes();
        if let Some(byte) = damaged.last_mut() {
            *byte ^= 0x1;
        }
        if let Ok(parsed) = String::from_utf8(damaged) {
            let _ = DocumentSnapshot::from_json(&parsed, &limits);
        }
    }
}

struct CacheProbe;

impl CacheProbe {
    fn run(engine: &Engine, salt: u8) {
        let dir = tempfile::tempdir().expect("cache directory");
        let path = dir.path().join("index.cache");
        let cache = airust::cache::IndexCache::new(
            &path,
            airust::cache::CacheIdentity::new("fuzz-seeds").expect("cache identity"),
        )
        .with_limits(airust::cache::CacheLimits::new(1_048_576, 10_000).expect("cache limits"));
        let entries: Vec<_> = engine.entries().cloned().collect();
        if cache.save(engine).is_err() {
            return;
        }
        let mut originals = entries.clone();
        let loaded = cache.load(&mut originals).expect("saved cache loads");
        assert!(originals.is_empty());
        let options = SearchOptions {
            limit: 4,
            ..SearchOptions::default()
        };
        assert_eq!(
            Formula::signature(&loaded.index().search_with_options("qa", &options)),
            Formula::signature(&engine.search_with_options("qa", &options))
        );
        let mut bytes = std::fs::read(&path).expect("cache bytes");
        if bytes.len() <= 64 {
            return;
        }
        let index = 64 + (salt as usize % (bytes.len() - 64));
        bytes[index] ^= 0xff;
        match salt % 3 {
            0 => {
                std::fs::write(&path, &bytes).expect("damaged cache");
                let mut kept = entries.clone();
                let error = cache.load(&mut kept);
                assert!(error.is_err());
                assert_eq!(kept, entries);
            }
            1 => {
                let digest = Sha256::digest(&bytes[64..]);
                bytes[32..64].copy_from_slice(&digest);
                let length = (bytes.len() - 64) as u64;
                bytes[24..32].copy_from_slice(&length.to_le_bytes());
                std::fs::write(&path, &bytes).expect("rehashed cache");
                let mut kept = entries.clone();
                match cache.load(&mut kept) {
                    Ok(loaded) => {
                        assert!(kept.is_empty());
                        let _ = loaded.index().search("qa", 3);
                        let _ = loaded.index().stats();
                    }
                    Err(_) => assert_eq!(kept, entries),
                }
            }
            _ => {
                bytes[0] = 0;
                std::fs::write(&path, &bytes).expect("bad magic");
                let mut kept = entries;
                assert!(cache.load(&mut kept).is_err());
            }
        }
    }
}
