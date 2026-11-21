use airust::{text, Bm25Config, Context, Engine, KnowledgeEntry, SearchOptions};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

struct FullScan {
    entries: Vec<KnowledgeEntry>,
    terms: Vec<BTreeMap<String, usize>>,
    frequencies: BTreeMap<String, usize>,
    tokens: usize,
}
impl FullScan {
    fn new(entries: Vec<KnowledgeEntry>) -> Self {
        let terms: Vec<_> = entries
            .iter()
            .map(|e| text::term_frequencies(&e.text))
            .collect();
        let mut frequencies = BTreeMap::new();
        let mut tokens = 0;
        for row in &terms {
            tokens += row.values().sum::<usize>();
            for word in row.keys() {
                *frequencies.entry(word.clone()).or_default() += 1;
            }
        }
        Self {
            entries,
            terms,
            frequencies,
            tokens,
        }
    }
    fn query(&self, query: &str, options: &SearchOptions) -> Vec<(String, u64, Vec<String>)> {
        let query = text::unique_terms(query);
        let count = self.entries.len();
        let average = self.tokens as f64 / count as f64;
        let mut hits = Vec::new();
        for (entry, terms) in self.entries.iter().zip(&self.terms) {
            if options
                .context
                .as_ref()
                .is_some_and(|scope| !scope.includes(entry))
            {
                continue;
            }
            let length = terms.values().sum::<usize>();
            let matched: Vec<_> = query
                .iter()
                .filter(|term| terms.contains_key(*term))
                .cloned()
                .collect();
            let score: f64 = matched
                .iter()
                .map(|term| {
                    let tf = terms[term] as f64;
                    let df = self.frequencies[term] as f64;
                    let idf = (1.0 + (count as f64 - df + 0.5) / (df + 0.5)).ln();
                    let norm = 1.0 - options.bm25.b() + options.bm25.b() * length as f64 / average;
                    if options.bm25.k1() >= 1.0 {
                        idf * tf * (1.0 + 1.0 / options.bm25.k1()) / (norm + tf / options.bm25.k1())
                    } else {
                        idf * tf * (options.bm25.k1() + 1.0) / (tf + options.bm25.k1() * norm)
                    }
                })
                .sum();
            if score > 0.0 && score.is_finite() {
                hits.push((entry.id.clone(), score, matched));
            }
        }
        hits.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        hits.truncate(options.limit);
        hits.into_iter()
            .map(|(id, score, terms)| (id, score.to_bits(), terms))
            .collect()
    }
}
fn actual(index: &Engine, query: &str, options: &SearchOptions) -> Vec<(String, u64, Vec<String>)> {
    index
        .search_with_options(query, options)
        .into_iter()
        .map(|h| (h.entry.id.clone(), h.score.to_bits(), h.matched_terms))
        .collect()
}

#[test]
fn prepared_bm25_matches_full_scan_score_bits_with_extreme_parameters_and_scopes() {
    let words = ["alpha", "beta", "café", "東京", "गुरु", "123", "a-2", "gamma"];
    let entries: Vec<_> = (0..64)
        .map(|n| {
            let body = (0..(1 + n % 23))
                .map(|i| words[(i * i + n * 3 + i / 2) % words.len()])
                .collect::<Vec<_>>()
                .join(" ");
            KnowledgeEntry::new(format!("id-{n:03}"), body).with_metadata(json!({"group": n%3}))
        })
        .chain([KnowledgeEntry::new("punctuation", "😀 ---")])
        .collect();
    let oracle = FullScan::new(entries.clone());
    let index = Engine::from_entries(entries).unwrap();
    for (k1, b) in [
        (f64::MIN_POSITIVE, 0.0),
        (1e-200, 1.0),
        (0.9999999999999999, 0.75),
        (1.0, 0.0),
        (1.2, 0.75),
        (f64::MAX, 1.0),
        (f64::MAX, 0.0),
    ] {
        for context in [
            None,
            Some(Context::new().with_metadata("group", 1)),
            Some(Context::new().with_entry("id-010").with_entry("id-003")),
            Some(Context::new().with_entry("absent")),
        ] {
            for limit in [0, 1, 7, 65, usize::MAX] {
                let options = SearchOptions {
                    bm25: Bm25Config::new(k1, b).unwrap(),
                    context: context.clone(),
                    limit,
                    ..Default::default()
                };
                for query in [
                    "",
                    "missing",
                    "alpha alpha beta absent",
                    "CAFÉ गुरु",
                    "a-2 123 東京",
                    "--- 😀",
                ] {
                    assert_eq!(
                        actual(&index, query, &options),
                        oracle.query(query, &options),
                        "k1={k1} b={b} limit={limit}"
                    );
                }
            }
        }
    }
}

#[test]
fn sorted_token_prefix_and_streaming_tail_preserve_counts_unicode_and_updates() {
    let mut entries = Vec::new();
    for n in [0, 1, 31, 1023, 1024, 1025, 2049, 12000] {
        let mut words = (0..n)
            .map(|i| {
                if i % 3 == 0 {
                    format!("unique{i}")
                } else {
                    "ＣＡＦÉ".into()
                }
            })
            .collect::<Vec<_>>();
        words.push("尾部".into());
        words.push("tailafterbatch".into());
        entries.push(KnowledgeEntry::new(
            format!("boundary-{n}"),
            words.join(" "),
        ));
    }
    let mut index = Engine::from_entries(entries.clone()).unwrap();
    for step in 0..3 {
        let oracle = FullScan::new(entries.clone());
        assert_eq!(index.stats().tokens, oracle.tokens);
        assert_eq!(index.stats().terms, oracle.frequencies.len());
        for query in [
            "café 尾部",
            "unique1023 unique2046 tailafterbatch",
            "unique0 unique11997",
        ] {
            let options = SearchOptions {
                limit: usize::MAX,
                ..Default::default()
            };
            assert_eq!(
                actual(&index, query, &options),
                oracle.query(query, &options)
            );
        }
        let restored = Engine::from_json(&index.to_json().unwrap()).unwrap();
        assert_eq!(restored.to_json().unwrap(), index.to_json().unwrap());
        if step == 0 {
            let changed = KnowledgeEntry::new("boundary-1024", "café café replacement");
            index.upsert(changed.clone()).unwrap();
            let position = entries.iter().position(|e| e.id == changed.id).unwrap();
            entries[position] = changed;
        } else if step == 1 {
            index.remove("boundary-12000");
            entries.retain(|e| e.id != "boundary-12000");
        }
    }
}

#[test]
fn deferred_explanations_keep_all_ties_and_ignore_unknown_query_terms() {
    let entries: Vec<_> = ["z", "a", "原", "a\nraw"]
        .into_iter()
        .map(|id| KnowledgeEntry::new(id, "beta alpha alpha"))
        .collect();
    let index = Engine::from_entries(entries.clone()).unwrap();
    let oracle = FullScan::new(entries);
    for limit in [1, 2, 3, 4, 99] {
        let options = SearchOptions {
            limit,
            ..Default::default()
        };
        let hits = actual(&index, "unknown beta alpha alpha absent", &options);
        assert_eq!(
            hits,
            oracle.query("unknown beta alpha alpha absent", &options)
        );
        assert!(hits.iter().all(|h| h.2 == ["alpha", "beta"]));
        assert_eq!(
            hits.iter().map(|h| &h.0).collect::<BTreeSet<_>>().len(),
            hits.len()
        );
    }
}
