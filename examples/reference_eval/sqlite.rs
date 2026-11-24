use super::{
    models::{independent_cosine, Dense},
    Result,
};
use airust::{
    retrieval::{
        RankingEvidence, RankingKind, RetrievalBatch, RetrievalCandidate, RetrievalError,
        RetrievalOptions, RetrievalSource,
    },
    Context, KnowledgeEntry,
};
use rusqlite::{params, Connection};
use std::collections::{BTreeMap, BTreeSet};

pub struct SqlSource {
    connection: Connection,
    entries: BTreeMap<String, KnowledgeEntry>,
    collection: String,
    pub dense: Option<Dense>,
}

impl SqlSource {
    pub fn new(
        entries: Vec<KnowledgeEntry>,
        collection: &str,
        dense: Option<Dense>,
    ) -> Result<Self> {
        let mut connection = Connection::open_in_memory()?;
        connection.execute_batch("CREATE VIRTUAL TABLE search USING fts5(id UNINDEXED, collection UNINDEXED, terms, tokenize='ascii'); CREATE VIRTUAL TABLE vocabulary USING fts5vocab(search, 'row');")?;
        let mut expected = BTreeMap::<String, (i64, i64)>::new();
        {
            let transaction = connection.transaction()?;
            for entry in &entries {
                let terms = airust::text::tokenize(&entry.text);
                for (term, frequency) in airust::text::term_frequencies(&entry.text) {
                    let counts = expected.entry(term).or_default();
                    counts.0 += 1;
                    counts.1 += i64::try_from(frequency)?;
                }
                transaction.execute(
                    "INSERT INTO search(id,collection,terms) VALUES (?1,?2,?3)",
                    params![
                        entry.id,
                        entry
                            .metadata
                            .as_ref()
                            .and_then(|value| value["collection"].as_str())
                            .ok_or("missing collection")?,
                        terms.join(" ")
                    ],
                )?;
            }
            transaction.commit()?;
        }
        let actual: BTreeMap<_, _> = connection
            .prepare("SELECT term, doc, cnt FROM vocabulary")?
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    (row.get::<_, i64>(1)?, row.get::<_, i64>(2)?),
                ))
            })?
            .collect::<std::result::Result<_, _>>()?;
        if actual != expected {
            return Err(
                "SQLite token vocabulary or frequencies differ from core pretokenization".into(),
            );
        }
        Ok(Self {
            connection,
            entries: entries
                .into_iter()
                .map(|entry| (entry.id.clone(), entry))
                .collect(),
            collection: collection.to_owned(),
            dense,
        })
    }

    pub fn verify_dense_math(&self, queries: &[&str]) -> Result<Option<f64>> {
        let Some(dense) = &self.dense else {
            return Ok(None);
        };
        let mut maximum = 0.0f64;
        for query in queries {
            let vector = dense.query(query)?;
            for document in dense.documents.values() {
                let difference = (independent_cosine(vector.values(), document.values())
                    - vector.cosine(document)?)
                .abs();
                maximum = maximum.max(difference);
            }
        }
        if maximum > 1e-12 {
            return Err("independent dense cosine reference differs".into());
        }
        Ok(Some(maximum))
    }
}

impl RetrievalSource for SqlSource {
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        options: &RetrievalOptions,
    ) -> std::result::Result<RetrievalBatch<'a>, RetrievalError> {
        let execute = || -> Result<RetrievalBatch<'_>> {
            let base = Context::new().with_metadata("collection", self.collection.as_str());
            let scope = match options.context() {
                Some(context) => base.intersection(context),
                None => Some(base),
            };
            let Some(scope) = scope else {
                return Ok(RetrievalBatch::new("sqlite-empty-scope", Vec::new()));
            };
            let terms = airust::text::tokenize(query);
            let unique: BTreeSet<_> = terms.iter().cloned().collect();
            let syntax = terms
                .iter()
                .map(|term| format!("\"{term}\""))
                .collect::<Vec<_>>()
                .join(" OR ");
            let mut lexical = Vec::new();
            if !syntax.is_empty() {
                let mut statement = self.connection.prepare("SELECT id, -bm25(search) AS score FROM search WHERE search MATCH ?1 ORDER BY score DESC, id")?;
                let rows = statement.query_map([syntax], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
                })?;
                for row in rows {
                    let (id, score) = row?;
                    if scope.includes(&self.entries[&id]) {
                        lexical.push((id, score));
                        if lexical.len() == options.candidates() {
                            break;
                        }
                    }
                }
            }
            let recipe;
            let mut ranked = BTreeMap::<String, RankingEvidence>::new();
            if let Some(dense) = &self.dense {
                recipe = format!(
                    "sqlite-pretok-ascii-plus-independent-cosine-rrf-v1:{}",
                    dense.space.model()
                );
                let query_vector = dense.query(query)?;
                let mut semantic: Vec<_> = dense
                    .documents
                    .iter()
                    .filter(|(id, _)| scope.includes(&self.entries[*id]))
                    .map(|(id, vector)| {
                        (
                            id.clone(),
                            independent_cosine(query_vector.values(), vector.values()),
                        )
                    })
                    .collect();
                semantic.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                semantic.truncate(options.candidates());
                for (index, (id, score)) in lexical.iter().enumerate() {
                    let mut evidence = RankingEvidence::new(
                        RankingKind::Custom,
                        0.5 / (60.0 + index as f64 + 1.0),
                    );
                    evidence.lexical_rank = Some(index + 1);
                    evidence.lexical_score = Some(*score);
                    ranked.insert(id.clone(), evidence);
                }
                for (index, (id, score)) in semantic.into_iter().enumerate() {
                    let evidence = ranked
                        .entry(id)
                        .or_insert_with(|| RankingEvidence::new(RankingKind::Custom, 0.0));
                    evidence.score += 0.5 / (60.0 + index as f64 + 1.0);
                    evidence.semantic_rank = Some(index + 1);
                    evidence.semantic_score = Some(score);
                }
            } else {
                recipe = "sqlite-fts5-core-pretokenized-ascii-bm25-v1".to_owned();
                for (index, (id, score)) in lexical.into_iter().enumerate() {
                    let mut evidence = RankingEvidence::new(RankingKind::Custom, score);
                    evidence.lexical_rank = Some(index + 1);
                    evidence.lexical_score = Some(score);
                    ranked.insert(id, evidence);
                }
            }
            let mut candidates: Vec<_> = ranked
                .into_iter()
                .map(|(id, mut evidence)| {
                    let entry = &self.entries[&id];
                    let document_terms = airust::text::unique_terms(&entry.text);
                    evidence.matched_terms =
                        unique.intersection(&document_terms).cloned().collect();
                    RetrievalCandidate { entry, evidence }
                })
                .collect();
            candidates.sort_by(|a, b| {
                b.evidence
                    .score
                    .total_cmp(&a.evidence.score)
                    .then_with(|| a.entry.id.cmp(&b.entry.id))
            });
            candidates.truncate(options.candidates());
            Ok(RetrievalBatch::new(recipe, candidates))
        };
        execute().map_err(|error| RetrievalError::Source(error.to_string()))
    }
}
