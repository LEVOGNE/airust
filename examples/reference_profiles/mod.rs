//! Explicit local-model profiles. Loading never downloads model assets.
use crate::{
    pilot_support::write_new,
    reference_support::{bounded_read, encode_bounded, sha256, Result, Workspace, MAX_JSON},
};
use airust::{
    execution::WorkControl,
    hybrid::HybridSearchOptions,
    mcp::McpSearchBackend,
    reranking::{RerankOptions, Reranker},
    retrieval::{
        HybridSource, LexicalSource, RetrievalBatch, RetrievalError, RetrievalOptions,
        RetrievalResponse, RetrievalSource, Retriever,
    },
    semantic::{
        Embedding, EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace,
        SemanticIndex,
    },
    Engine, SearchAlgorithm, SearchOptions,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, TryLockError},
    time::Duration,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Bm25,
    Hybrid,
    HybridRerank,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailurePolicy {
    Error,
    LexicalFallback,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub url: String,
    pub model: String,
    pub digest: String,
    pub dimensions: usize,
    pub timeout_ms: u64,
}
impl Model {
    fn space(&self) -> Result<EmbeddingSpace> {
        let url = reqwest::Url::parse(&self.url)?;
        if url.scheme() != "http"
            || !matches!(
                url.host_str(),
                Some("127.0.0.1" | "localhost" | "[::1]" | "::1")
            )
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
            || self.model.trim().is_empty()
            || self.model.len() > 128
            || self.model.chars().any(char::is_control)
            || self.digest.len() != 64
            || !self
                .digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || self.dimensions == 0
            || self.dimensions > 4096
            || self.timeout_ms == 0
            || self.timeout_ms > 30_000
        {
            return Err(
                "invalid explicit local embedding endpoint, model identity or budget".into(),
            );
        }
        Ok(EmbeddingSpace::new(
            format!(
                "ollama:{}@{};raw;truncate=false;l2-f32-v1",
                self.model, self.digest
            ),
            self.dimensions,
        )?)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReranker {
    pub directory: PathBuf,
    pub runtime: PathBuf,
    pub threads: usize,
    #[serde(default = "default_rerank_batch_size")]
    pub batch_size: usize,
    #[serde(default)]
    pub candidates: Option<usize>,
}
fn default_rerank_batch_size() -> usize {
    16
}
impl NativeReranker {
    fn options(&self, limit: usize) -> std::result::Result<RerankOptions, RetrievalError> {
        Ok(RerankOptions::new(limit)
            .with_candidates(self.candidates.unwrap_or_else(|| limit.max(20)))?
            .with_batch_size(self.batch_size)?)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub profile: Kind,
    pub failure_policy: FailurePolicy,
    pub embedding: Option<Model>,
    pub vectors: Option<PathBuf>,
    pub reranker: Option<NativeReranker>,
}
impl Config {
    pub fn read(path: &Path) -> Result<Self> {
        let mut config: Self = serde_json::from_slice(&bounded_read(path, 16 * 1024)?)?;
        if config.schema_version != 1 {
            return Err("unsupported profile version".into());
        }
        let parent = path.parent().unwrap_or(Path::new("."));
        if let Some(p) = config.vectors.as_mut() {
            if p.is_relative() {
                *p = parent.join(&*p);
            }
        }
        if let Some(r) = config.reranker.as_mut() {
            if r.directory.is_relative() {
                r.directory = parent.join(&r.directory);
            }
            if r.runtime.is_relative() {
                r.runtime = parent.join(&r.runtime);
            }
            if r.threads == 0 || r.threads > 64 {
                return Err("reranker threads must be 1..64".into());
            }
            if r.batch_size > 16 || r.candidates.is_some_and(|n| n > 100) {
                return Err(
                    "reranker batch_size must be 1..16 and candidates must be 1..100".into(),
                );
            }
            r.options(0)?;
        }
        match config.profile {
            Kind::Bm25
                if config.embedding.is_none()
                    && config.vectors.is_none()
                    && config.reranker.is_none() => {}
            Kind::Hybrid
                if config.embedding.is_some()
                    && config.vectors.is_some()
                    && config.reranker.is_none() => {}
            Kind::HybridRerank
                if config.embedding.is_some()
                    && config.vectors.is_some()
                    && config.reranker.is_some() => {}
            _ => {
                return Err(
                    "profile and explicit model/vector/reranker configuration differ".into(),
                )
            }
        }
        if let Some(model) = &config.embedding {
            model.space()?;
        }
        Ok(config)
    }
}

struct Ollama {
    client: Client,
    model: Model,
    space: EmbeddingSpace,
    control: Option<WorkControl>,
}
impl Ollama {
    fn new(model: Model) -> Result<Self> {
        let space = model.space()?;
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_millis(model.timeout_ms))
            .build()?;
        Ok(Self {
            client,
            model,
            space,
            control: None,
        })
    }
    fn request(&self, path: &str, body: Option<Value>, maximum: usize) -> Result<Value> {
        if let Some(control) = &self.control {
            control.checkpoint()?;
        }
        let timeout = self
            .control
            .as_ref()
            .and_then(WorkControl::remaining)
            .unwrap_or(Duration::from_millis(self.model.timeout_ms))
            .min(Duration::from_millis(self.model.timeout_ms));
        if timeout.is_zero() {
            return Err("embedding deadline expired".into());
        }
        let url = format!("{}{}", self.model.url.trim_end_matches('/'), path);
        let request = if let Some(body) = body {
            self.client.post(url).json(&body)
        } else {
            self.client.get(url)
        };
        let response = request.timeout(timeout).send()?;
        if !response.status().is_success() {
            return Err(format!("local embedding service HTTP {}", response.status()).into());
        }
        let mut bytes = Vec::new();
        response.take(maximum as u64 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > maximum {
            return Err("embedding response byte budget exceeded".into());
        }
        if let Some(control) = &self.control {
            control.checkpoint()?;
        }
        Ok(serde_json::from_slice(&bytes)?)
    }
    fn verify(&self) -> Result<()> {
        let response = self.request("/api/tags", None, 2 * 1024 * 1024)?;
        let model = response["models"]
            .as_array()
            .and_then(|models| models.iter().find(|m| m["name"] == self.model.model))
            .ok_or("configured embedding model is unavailable")?;
        if model["digest"] != self.model.digest {
            return Err("embedding model digest changed".into());
        }
        Ok(())
    }
}
impl EmbeddingProvider for Ollama {
    fn space(&self) -> &EmbeddingSpace {
        &self.space
    }
    fn embed(
        &mut self,
        texts: &[&str],
        _purpose: EmbeddingPurpose,
    ) -> std::result::Result<Vec<Vec<f32>>, EmbeddingError> {
        let run = || -> Result<Vec<Vec<f32>>> {
            if texts.len() > 16 || texts.iter().map(|t| t.len()).sum::<usize>() > 64 * 1024 {
                return Err("embedding input batch budget exceeded".into());
            }
            self.verify()?;
            let result=self.request("/api/embed",Some(json!({"model":self.model.model,"input":texts,"truncate":false,"keep_alive":"10m"})),2*1024*1024)?;
            let vectors: Vec<Vec<f32>> = serde_json::from_value(result["embeddings"].clone())?;
            if vectors.len() != texts.len() {
                return Err("embedding count mismatch".into());
            }
            for vector in &vectors {
                Embedding::new(self.space.clone(), vector.clone())?;
            }
            self.verify()?;
            Ok(vectors)
        };
        run().map_err(|e| EmbeddingError::Provider(e.to_string()))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cache {
    schema_version: u32,
    workspace_sha256: String,
    embedding_recipe: String,
    dimensions: usize,
    vectors: BTreeMap<String, Vec<f32>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    payload_sha256: String,
    payload: String,
}

pub fn prepare(workspace: &Workspace, config_path: &Path, output: &Path) -> Result<Value> {
    if output.exists() {
        return Err("vector destination already exists".into());
    }
    let config = Config::read(config_path)?;
    let model = config
        .embedding
        .ok_or("a model profile is required to prepare vectors")?;
    let mut provider = Ollama::new(model)?;
    if workspace.index.len() > MAX_JSON / std::mem::size_of::<f32>() / provider.space.dimensions() {
        return Err("raw vector values exceed the 64 MiB preparation budget".into());
    }
    let entries: Vec<_> = workspace.index.entries().collect();
    let mut vectors = BTreeMap::new();
    let mut deduplicated = BTreeMap::<String, Vec<f32>>::new();
    for batch in entries.chunks(16) {
        let fresh: Vec<_> = batch
            .iter()
            .filter(|entry| !deduplicated.contains_key(&entry.text))
            .map(|entry| entry.text.as_str())
            .collect();
        if !fresh.is_empty() {
            for (text, values) in fresh
                .iter()
                .zip(provider.embed(&fresh, EmbeddingPurpose::Document)?)
            {
                deduplicated.insert((*text).to_owned(), values);
            }
        }
        for entry in batch {
            vectors.insert(entry.id.clone(), deduplicated[&entry.text].clone());
        }
    }
    let cache = Cache {
        schema_version: 1,
        workspace_sha256: workspace.identity(),
        embedding_recipe: provider.space.model().into(),
        dimensions: provider.space.dimensions(),
        vectors,
    };
    let payload = String::from_utf8(encode_bounded(&cache, MAX_JSON)?)?;
    let encoded = encode_bounded(
        &Envelope {
            payload_sha256: sha256(payload.as_bytes()),
            payload,
        },
        MAX_JSON,
    )?;
    write_new(output, &encoded)?;
    Ok(
        json!({"vectors":cache.vectors.len(),"unique_texts":deduplicated.len(),"workspace_sha256":cache.workspace_sha256,"embedding_recipe":cache.embedding_recipe,"bytes":encoded.len()}),
    )
}

struct State {
    semantic: Option<SemanticIndex>,
    provider: Option<Ollama>,
    reranker: Option<Box<dyn Reranker + Send>>,
}
pub struct Profile {
    config: Config,
    recipe: String,
    state: Mutex<State>,
}
struct Tagged<S> {
    source: S,
    recipe: String,
    fallback: Option<String>,
}
impl<S: RetrievalSource> RetrievalSource for Tagged<S> {
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        options: &RetrievalOptions,
    ) -> std::result::Result<RetrievalBatch<'a>, RetrievalError> {
        let mut batch = self.source.retrieve(query, options)?;
        batch.recipe = self.recipe.clone();
        if self.fallback.is_some() {
            batch.fallback_reason = self.fallback.clone();
        }
        Ok(batch)
    }
}
impl Profile {
    pub fn load(workspace: &Workspace, path: &Path) -> Result<Self> {
        let config = Config::read(path)?;
        if config.profile == Kind::Bm25 {
            return Ok(Self {
                config,
                recipe: "airust-pilot-bm25-v1".into(),
                state: Mutex::new(State {
                    semantic: None,
                    provider: None,
                    reranker: None,
                }),
            });
        }
        let provider = Ollama::new(
            config
                .embedding
                .clone()
                .ok_or("missing embedding configuration")?,
        )?;
        let envelope: Envelope = serde_json::from_slice(&bounded_read(
            config.vectors.as_deref().ok_or("missing vector path")?,
            MAX_JSON,
        )?)?;
        if sha256(envelope.payload.as_bytes()) != envelope.payload_sha256 {
            return Err("vector cache checksum mismatch".into());
        }
        let cache: Cache = serde_json::from_str(&envelope.payload)?;
        if cache.schema_version != 1
            || cache.workspace_sha256 != workspace.identity()
            || cache.embedding_recipe != provider.space.model()
            || cache.dimensions != provider.space.dimensions()
            || cache.vectors.len() != workspace.index.len()
        {
            return Err("vector cache workspace, model recipe or count mismatch".into());
        }
        let mut semantic = SemanticIndex::new(provider.space.clone());
        for entry in workspace.index.entries() {
            let values = cache
                .vectors
                .get(&entry.id)
                .ok_or("missing document vector")?;
            semantic.upsert_embedded(
                entry.clone(),
                Embedding::new(provider.space.clone(), values.clone())?,
            )?;
        }
        let reranker = match &config.reranker {
            Some(paths) => Some(native(paths)?),
            None => None,
        };
        let recipe = format!(
            "airust-pilot-{:?}-v1;{}",
            config.profile,
            provider.space.model()
        );
        Ok(Self {
            config,
            recipe,
            state: Mutex::new(State {
                semantic: Some(semantic),
                provider: Some(provider),
                reranker,
            }),
        })
    }
    fn lock(
        &self,
        options: &RetrievalOptions,
    ) -> std::result::Result<MutexGuard<'_, State>, RetrievalError> {
        loop {
            if let Some(control) = options.control() {
                control.checkpoint()?;
            }
            match self.state.try_lock() {
                Ok(guard) => return Ok(guard),
                Err(TryLockError::Poisoned(_)) => {
                    return Err(RetrievalError::Source("model state is poisoned".into()))
                }
                Err(TryLockError::WouldBlock) => {
                    std::thread::park_timeout(Duration::from_millis(1))
                }
            }
        }
    }
    pub fn query(
        &self,
        index: &Engine,
        query: &str,
        algorithm: SearchAlgorithm,
        options: &RetrievalOptions,
    ) -> std::result::Result<RetrievalResponse, RetrievalError> {
        if self.config.profile == Kind::Bm25 {
            return Retriever::new(Tagged {
                source: LexicalSource::new(index).with_options(SearchOptions {
                    algorithm,
                    ..Default::default()
                }),
                recipe: if algorithm == SearchAlgorithm::Tfidf {
                    "airust-pilot-tfidf-v1".into()
                } else {
                    self.recipe.clone()
                },
                fallback: None,
            })
            .query(query, options);
        }
        if algorithm != SearchAlgorithm::Bm25 {
            return Err(RetrievalError::Configuration(
                "model profiles require BM25 fusion; TF-IDF is an explicit lexical profile",
            ));
        }
        if let Some(control) = options.control() {
            control.checkpoint()?;
        }
        let rerank_options = self
            .config
            .reranker
            .as_ref()
            .map(|config| config.options(options.limit()))
            .transpose()?;
        let mut state = self.lock(options)?;
        let State {
            semantic,
            provider,
            reranker,
        } = &mut *state;
        let semantic = semantic
            .as_ref()
            .ok_or(RetrievalError::Configuration("missing semantic index"))?;
        if semantic.lexical().stats() != index.stats() {
            return Err(RetrievalError::Output("profile and server index differ"));
        }
        let provider = provider
            .as_mut()
            .ok_or(RetrievalError::Configuration("missing embedding provider"))?;
        provider.control = options.control().cloned();
        let source = Tagged {
            source: HybridSource::new(semantic, provider)
                .with_options(HybridSearchOptions::new(options.limit())),
            recipe: self.recipe.clone(),
            fallback: None,
        };
        let mut retriever = Retriever::new(source);
        if let (Some(model), Some(rerank_options)) = (reranker.as_mut(), rerank_options) {
            retriever = retriever.with_reranker(model.as_mut(), rerank_options);
        }
        let result = retriever.query(query, options);
        drop(retriever);
        // Model state outlives individual requests; release their cancellation
        // and observer references as soon as inference returns.
        provider.control = None;
        if let Some(control) = options.control() {
            control.checkpoint()?;
        }
        match result {
            Err(error @ (RetrievalError::Source(_) | RetrievalError::Rerank(_)))
                if self.config.failure_policy == FailurePolicy::LexicalFallback =>
            {
                let reason = format!(
                    "explicit lexical fallback from {:?}: {error}",
                    self.config.profile
                );
                Retriever::new(Tagged {
                    source: LexicalSource::new(index),
                    recipe: format!("airust-pilot-lexical-fallback-v1;{}", self.recipe),
                    fallback: Some(reason),
                })
                .query(query, options)
            }
            other => other,
        }
    }
}
impl McpSearchBackend for Profile {
    fn retrieve(
        &self,
        index: &Engine,
        query: &str,
        algorithm: SearchAlgorithm,
        options: &RetrievalOptions,
    ) -> std::result::Result<RetrievalResponse, RetrievalError> {
        self.query(index, query, algorithm, options)
    }
}
fn native(paths: &NativeReranker) -> Result<Box<dyn Reranker + Send>> {
    #[cfg(feature = "local-reranking")]
    {
        Ok(Box::new(
            airust::reranking::local::FastEmbedReranker::multilingual_minilm_v2(
                &paths.directory,
                &paths.runtime,
                paths.threads,
            )?,
        ))
    }
    #[cfg(not(feature = "local-reranking"))]
    {
        let _ = paths;
        Err("enable local-reranking and supply the verified native assets explicitly".into())
    }
}

#[cfg(test)]
mod configuration_tests {
    use super::*;
    use airust::{execution::CancellationToken, reranking::RerankError};

    fn profile() -> Value {
        json!({
            "schema_version": 1, "profile": "hybrid_rerank", "failure_policy": "error",
            "embedding": {"url": "http://127.0.0.1:1", "model": "fixture:local", "digest": "1".repeat(64), "dimensions": 3, "timeout_ms": 1000},
            "vectors": "missing-vectors.json",
            "reranker": {"directory": "missing-model", "runtime": "missing-runtime", "threads": 2},
        })
    }
    fn read(value: &Value) -> Config {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");
        std::fs::write(&path, serde_json::to_vec(value).unwrap()).unwrap();
        Config::read(&path).unwrap()
    }

    #[test]
    fn existing_profiles_keep_batch_and_automatic_candidate_defaults() {
        let config = read(&profile());
        let native = config.reranker.unwrap();
        assert_eq!(native.batch_size, 16);
        assert_eq!(native.candidates, None);
        for limit in [0, 6, 20, 25, 100] {
            let options = native.options(limit).unwrap();
            assert_eq!(options.limit(), limit);
            assert_eq!(options.batch_size(), 16);
            assert_eq!(options.candidates(), limit.max(20));
        }
    }

    #[test]
    fn explicit_batch_and_candidate_budget_reuse_validated_reranking_options() {
        let mut value = profile();
        value["reranker"]["batch_size"] = json!(1);
        value["reranker"]["candidates"] = json!(20);
        let config = read(&value);
        let native = config.reranker.unwrap();
        let options = native.options(6).unwrap();
        assert_eq!(options.candidates(), 20);
        assert_eq!(options.batch_size(), 1);
        assert!(matches!(
            native.options(21),
            Err(RetrievalError::Rerank(RerankError::Configuration(_)))
        ));
    }

    #[test]
    fn invalid_native_bounds_and_unknown_fields_fail_without_loading_assets() {
        for (name, invalid) in [
            ("batch_size", json!(0)),
            ("batch_size", json!(17)),
            ("candidates", json!(0)),
            ("candidates", json!(101)),
            ("threads", json!(0)),
            ("threads", json!(65)),
            ("batch_size", json!(1.5)),
            ("candidates", json!(-1)),
            ("batch_szie", json!(1)),
        ] {
            let mut value = profile();
            value["reranker"][name] = invalid;
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("profile.json");
            let original = serde_json::to_vec(&value).unwrap();
            std::fs::write(&path, &original).unwrap();
            assert!(Config::read(&path).is_err(), "{name}");
            assert_eq!(std::fs::read(&path).unwrap(), original);
        }
    }

    #[test]
    fn preflight_resolves_relative_paths_without_accessing_cache_model_or_service() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");
        std::fs::write(&path, serde_json::to_vec(&profile()).unwrap()).unwrap();
        let config = Config::read(&path).unwrap();
        assert_eq!(
            config.vectors.unwrap(),
            dir.path().join("missing-vectors.json")
        );
        let native = config.reranker.unwrap();
        assert_eq!(native.directory, dir.path().join("missing-model"));
        assert_eq!(native.runtime, dir.path().join("missing-runtime"));
        assert!(!native.directory.exists() && !native.runtime.exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    fn unloaded(config: Config) -> Profile {
        Profile {
            config,
            recipe: "not-loaded".into(),
            state: Mutex::new(State {
                semantic: None,
                provider: None,
                reranker: None,
            }),
        }
    }

    #[test]
    fn request_cannot_expand_explicit_window_or_turn_configuration_into_fallback() {
        let mut value = profile();
        value["failure_policy"] = json!("lexical_fallback");
        value["reranker"]["candidates"] = json!(5);
        let profile = unloaded(read(&value));
        assert!(matches!(
            profile.query(
                &Engine::new(),
                "Rust",
                SearchAlgorithm::Bm25,
                &RetrievalOptions::new(6)
            ),
            Err(RetrievalError::Rerank(RerankError::Configuration(_)))
        ));
    }

    #[test]
    fn cancellation_retains_priority_over_invalid_explicit_window() {
        let mut value = profile();
        value["reranker"]["candidates"] = json!(5);
        let profile = unloaded(read(&value));
        let cancel = CancellationToken::new();
        cancel.cancel();
        let options = RetrievalOptions::new(6).with_control(WorkControl::new(cancel));
        assert!(matches!(
            profile.query(&Engine::new(), "Rust", SearchAlgorithm::Bm25, &options),
            Err(RetrievalError::Execution(
                airust::execution::ExecutionError::Cancelled
            ))
        ));
    }

    #[test]
    fn expired_deadline_retains_priority_over_invalid_explicit_window() {
        let mut value = profile();
        value["reranker"]["candidates"] = json!(5);
        let profile = unloaded(read(&value));
        let control =
            WorkControl::new(CancellationToken::new()).with_deadline(std::time::Instant::now());
        let options = RetrievalOptions::new(6).with_control(control);
        assert!(matches!(
            profile.query(&Engine::new(), "Rust", SearchAlgorithm::Bm25, &options),
            Err(RetrievalError::Execution(
                airust::execution::ExecutionError::DeadlineExceeded
            ))
        ));
    }
}
