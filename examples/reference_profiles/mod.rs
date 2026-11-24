//! Explicit local-model profiles. Loading never downloads model assets.
use crate::{
    pilot_support::write_new,
    reference_support::{bounded_read, sha256, Result, Workspace, MAX_JSON},
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
    let payload = serde_json::to_string(&cache)?;
    let encoded = serde_json::to_vec(&Envelope {
        payload_sha256: sha256(payload.as_bytes()),
        payload,
    })?;
    if encoded.len() > MAX_JSON {
        return Err("vector cache exceeds 64 MiB".into());
    }
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
                recipe: self.recipe.clone(),
                fallback: None,
            })
            .query(query, options);
        }
        if algorithm != SearchAlgorithm::Bm25 {
            return Err(RetrievalError::Configuration(
                "model profiles require BM25 fusion; TF-IDF is an explicit lexical profile",
            ));
        }
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
        if let Some(model) = reranker.as_mut() {
            retriever = retriever.with_reranker(
                model.as_mut(),
                RerankOptions::new(options.limit()).with_candidates(options.limit().max(20))?,
            );
        }
        let result = retriever.query(query, options);
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
