#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod persistence;
mod ranking;

#[cfg(any(feature = "local-embeddings", feature = "local-reranking"))]
mod local_runtime;

#[cfg(feature = "cache")]
pub mod cache;

pub mod context;
#[cfg(feature = "context-assembly")]
pub mod context_engine;
#[cfg(feature = "graph")]
pub mod graph;
#[cfg(feature = "hybrid")]
pub mod hybrid;
pub mod index;
pub mod matching;
#[cfg(feature = "provenance")]
pub mod provenance;
pub mod search;
#[cfg(feature = "embeddings")]
pub mod semantic;
pub mod text;

pub mod agent;
pub mod context_agent;
pub mod knowledge;
pub mod match_agent;
#[cfg(feature = "pdf")]
pub mod pdf_loader;
pub mod tfidf_agent;

#[cfg(feature = "web")]
pub mod web;

pub use agent::ConfidenceAgent;
pub use agent::{
    Agent, AgentError, ContextualAgent, ResponseFormat, TrainableAgent, TrainingExample,
};
pub use context::Context;
pub use index::{IndexError, IndexStats, KnowledgeIndex};
pub use knowledge::{KnowledgeEntry, RelationKind, RelationSuggestion};
pub use matching::{compare, MatchError, MatchOptions, MatchResult, MatchStrategy, Similarity};
pub use search::{Bm25Config, SearchAlgorithm, SearchError, SearchOptions, SearchResult};

pub type Engine = KnowledgeIndex;
pub use context_agent::ContextAgent;
pub use knowledge::KnowledgeBase;
pub use match_agent::MatchAgent;
#[cfg(feature = "pdf")]
pub use pdf_loader::{
    pdf_to_knowledge_base, pdf_to_training_examples, PageContent, PdfLoader, PdfLoaderConfig,
};
pub use tfidf_agent::TfidfAgent;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

pub fn version_info() -> String {
    format!("{} v{}", NAME, VERSION)
}

pub fn library_description() -> String {
    "AIRust: A Modular Knowledge Intelligence Engine — find knowledge, compare text, suggest connections".to_string()
}

#[cfg(feature = "hnsw")]
pub mod ann;

#[cfg(feature = "versioning")]
pub mod versioning;

#[cfg(feature = "quality")]
pub mod quality;

#[cfg(feature = "reranking")]
pub mod reranking;

#[cfg(feature = "advanced-search")]
pub mod text_search;

#[cfg(feature = "storage")]
pub mod storage;

#[cfg(feature = "retrieval")]
pub mod retrieval;

#[cfg(feature = "snapshots")]
pub mod snapshots;

#[cfg(feature = "execution")]
pub mod execution;

#[cfg(feature = "adaptive")]
pub mod adaptive;

#[cfg(feature = "feedback")]
pub mod feedback;

#[cfg(feature = "learned-ranking")]
pub mod learned_ranking;
