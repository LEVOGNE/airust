use airust::{
    reranking::{RerankOptions, Reranker},
    retrieval::{
        ExcerptOptions, HybridSource, LexicalCoverageSource, LexicalQueryOptions, LexicalSource,
        RetrievalOptions, RetrievalResponse, RetrievalSource, Retriever, TextSource,
    },
    semantic::{Embedding, SemanticIndex},
    text_search::{AnalyzerConfig, TextIndex},
    Context, Engine, KnowledgeEntry,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, error::Error, fs::File, io::Read, path::Path};

pub mod models;
mod sqlite;
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
pub struct Row {
    pub id: String,
    pub document: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub collection: String,
}

#[derive(Deserialize)]
pub struct Case {
    pub id: String,
    pub family: String,
    pub query: String,
    pub split: String,
    pub language: String,
    pub answerable: bool,
    pub reference_quote: Option<String>,
}

#[derive(Deserialize)]
pub struct Judgment {
    pub query_id: String,
    pub document_id: String,
    pub grade: u8,
}

#[derive(Deserialize)]
struct Document {
    id: String,
    text: String,
    sha256: String,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn read(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err("evaluation input exceeds 64 MiB".into());
    }
    Ok(bytes)
}

pub fn lines<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    std::str::from_utf8(&read(path)?)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

pub struct Dataset {
    pub manifest: Value,
    pub policy: Value,
    pub rows: BTreeMap<String, Row>,
    pub cases: Vec<Case>,
    pub judgments: Vec<Judgment>,
    documents: BTreeMap<String, String>,
}

impl Dataset {
    pub fn load(directory: &Path, split: &str) -> Result<Self> {
        if !matches!(split, "development" | "evaluation") {
            return Err("unknown dataset split".into());
        }
        let manifest: Value = serde_json::from_slice(&read(&directory.join("manifest.json"))?)?;
        for name in [
            "documents.json",
            "corpus.jsonl",
            "cases.jsonl",
            "qrels.jsonl",
        ] {
            if digest(&read(&directory.join(name))?)
                != manifest["artifacts"][name]["sha256"]
                    .as_str()
                    .ok_or("missing input identity")?
            {
                return Err(format!("export identity mismatch: {name}").into());
            }
        }
        let policy_bytes = read(&directory.join("policy.json"))?;
        if digest(&policy_bytes)
            != manifest["inputs"]["policy_sha256"]
                .as_str()
                .ok_or("missing policy identity")?
        {
            return Err("acceptance policy identity mismatch".into());
        }
        let policy = serde_json::from_slice(&policy_bytes)?;
        let documents: Vec<Document> =
            serde_json::from_slice(&read(&directory.join("documents.json"))?)?;
        let mut source = BTreeMap::new();
        for document in documents {
            if digest(document.text.as_bytes()) != document.sha256
                || source.insert(document.id, document.text).is_some()
            {
                return Err("source document identity mismatch".into());
            }
        }
        let mut rows = BTreeMap::new();
        for row in lines::<Row>(&directory.join("corpus.jsonl"))? {
            if source
                .get(&row.document)
                .and_then(|text| text.get(row.start..row.end))
                != Some(row.text.as_str())
            {
                return Err("corpus passage is not an exact UTF-8 source span".into());
            }
            if rows.insert(row.id.clone(), row).is_some() {
                return Err("duplicate corpus ID".into());
            }
        }
        let cases = lines::<Case>(&directory.join("cases.jsonl"))?
            .into_iter()
            .filter(|case| case.split == split)
            .collect();
        Ok(Self {
            manifest,
            policy,
            rows,
            cases,
            judgments: lines(&directory.join("qrels.jsonl"))?,
            documents: source,
        })
    }

    pub fn validate_response(&self, response: &RetrievalResponse, collection: &str) -> Result<()> {
        for hit in response.hits() {
            let row = self.rows.get(&hit.id).ok_or("returned unknown corpus ID")?;
            if row.collection != collection
                || row.text.get(hit.span.start..hit.span.end) != Some(hit.excerpt.as_str())
            {
                return Err("scope or entry-relative span violation".into());
            }
            let start = row
                .start
                .checked_add(hit.span.start)
                .ok_or("span overflow")?;
            let end = row.start.checked_add(hit.span.end).ok_or("span overflow")?;
            if self.documents[&row.document].get(start..end) != Some(hit.excerpt.as_str()) {
                return Err("global source projection violation".into());
            }
        }
        Ok(())
    }
}

pub struct Indexes {
    pub profile: String,
    collection: String,
    lexical: Option<Engine>,
    text: Option<TextIndex>,
    sql: Option<sqlite::SqlSource>,
    semantic: Option<SemanticIndex>,
    provider: Option<models::QueryProvider>,
    reranker: Option<Box<dyn Reranker>>,
}

impl Indexes {
    pub fn build(
        dataset: &Dataset,
        profile: &str,
        vector_path: Option<&Path>,
        reranker_paths: Option<(&Path, &Path)>,
    ) -> Result<Self> {
        if ![
            "bm25",
            "prepared",
            "coverage",
            "german",
            "sqlite",
            "hybrid",
            "sqlite_hybrid",
            "bm25_rerank",
            "hybrid_rerank",
        ]
        .contains(&profile)
        {
            return Err("unknown retrieval profile".into());
        }
        let collection = dataset.policy["corpus"]["collection"]
            .as_str()
            .ok_or("missing collection")?
            .to_owned();
        let entries: Vec<_> = dataset
            .rows
            .values()
            .map(|row| {
                KnowledgeEntry::new(&row.id, &row.text)
                    .with_metadata(serde_json::json!({"collection": row.collection}))
            })
            .collect();
        let vectors = if profile.contains("hybrid") {
            Some(models::Vectors::load(
                vector_path.ok_or("hybrid profile requires explicit vectors")?,
                dataset.manifest["identity_sha256"]
                    .as_str()
                    .ok_or("dataset identity")?,
                dataset.manifest["artifacts"]["corpus.jsonl"]["sha256"]
                    .as_str()
                    .ok_or("corpus identity")?,
            )?)
        } else {
            None
        };
        let mut result = Self {
            profile: profile.to_owned(),
            collection: collection.clone(),
            lexical: None,
            text: None,
            sql: None,
            semantic: None,
            provider: None,
            reranker: None,
        };
        if profile.starts_with("sqlite") {
            result.sql = Some(sqlite::SqlSource::new(
                entries,
                &collection,
                vectors.map(models::Dense::from_vectors).transpose()?,
            )?);
        } else if profile == "german" {
            result.text = Some(TextIndex::from_entries(AnalyzerConfig::german(), entries)?);
        } else if let Some(vectors) = vectors {
            let space = vectors.space()?;
            let mut index = SemanticIndex::new(space.clone());
            for entry in entries {
                let vector = vectors
                    .documents
                    .get(&entry.id)
                    .ok_or("document vector is missing")?
                    .clone();
                index.upsert_embedded(entry, Embedding::new(space.clone(), vector)?)?;
            }
            result.provider = Some(models::QueryProvider {
                space,
                queries: vectors.queries,
            });
            result.semantic = Some(index);
        } else {
            result.lexical = Some(Engine::from_entries(entries)?);
        }
        if profile.ends_with("rerank") {
            let (directory, runtime) = reranker_paths
                .ok_or("reranking requires explicit verified model and runtime paths")?;
            result.reranker = Some(models::native_reranker(directory, runtime)?);
        }
        Ok(result)
    }

    pub fn verify_dense_reference(&self, queries: &[&str]) -> Result<Option<f64>> {
        self.sql
            .as_ref()
            .map_or(Ok(None), |source| source.verify_dense_math(queries))
    }

    pub fn query(&mut self, query: &str, limit: usize) -> Result<RetrievalResponse> {
        let source: Box<dyn RetrievalSource + '_> = match self.profile.as_str() {
            "bm25" | "bm25_rerank" => Box::new(LexicalSource::new(
                self.lexical.as_ref().ok_or("missing lexical index")?,
            )),
            "prepared" => Box::new(
                LexicalSource::new(self.lexical.as_ref().ok_or("missing lexical index")?)
                    .with_query_analysis(LexicalQueryOptions::default()),
            ),
            "coverage" => Box::new(LexicalCoverageSource::new(
                self.lexical.as_ref().ok_or("missing lexical index")?,
            )),
            "german" => Box::new(TextSource::new(
                self.text.as_ref().ok_or("missing positional index")?,
            )),
            "sqlite" | "sqlite_hybrid" => {
                Box::new(self.sql.as_mut().ok_or("missing SQLite index")?)
            }
            "hybrid" | "hybrid_rerank" => Box::new(HybridSource::new(
                self.semantic.as_ref().ok_or("missing semantic index")?,
                self.provider.as_mut().ok_or("missing query vectors")?,
            )),
            _ => return Err("unknown retrieval profile".into()),
        };
        let options = RetrievalOptions::new(limit)
            .with_candidates(100)?
            .with_context(Context::new().with_metadata("collection", self.collection.as_str()))
            .with_byte_budgets(4096, 12000, 2000, 65536)?
            .with_relevant_excerpts(ExcerptOptions::default().with_paragraph_boundaries())
            .with_distinct_evidence();
        let mut retriever = Retriever::new(source);
        if let Some(scorer) = self.reranker.as_mut() {
            retriever = retriever.with_reranker(
                scorer.as_mut(),
                RerankOptions::new(limit).with_candidates(20)?,
            );
        }
        Ok(retriever.query(query, &options)?)
    }
}
