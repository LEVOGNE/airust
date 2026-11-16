#![doc = include_str!("../README.md")]

pub mod context;
pub mod index;
pub mod matching;
pub mod search;
pub mod text;

// Compatibility adapters for the existing training and agent API.
pub mod agent;
pub mod context_agent;
pub mod knowledge;
pub mod match_agent;
#[cfg(feature = "pdf")]
pub mod pdf_loader;
pub mod tfidf_agent;

#[cfg(feature = "web")]
pub mod web;

// Re-exports for easier usage
pub use agent::ConfidenceAgent;
pub use agent::{
    Agent, AgentError, ContextualAgent, ResponseFormat, TrainableAgent, TrainingExample,
};
pub use context::Context;
pub use index::{IndexError, IndexStats, KnowledgeIndex};
pub use knowledge::{KnowledgeEntry, RelationKind, RelationSuggestion};
pub use matching::{compare, MatchError, MatchOptions, MatchResult, MatchStrategy, Similarity};
pub use search::{Bm25Config, SearchAlgorithm, SearchError, SearchOptions, SearchResult};

/// The shared index and its retrieval operations.
pub type Engine = KnowledgeIndex;
pub use context_agent::ContextAgent;
pub use knowledge::KnowledgeBase;
pub use match_agent::MatchAgent;
#[cfg(feature = "pdf")]
pub use pdf_loader::{
    pdf_to_knowledge_base, pdf_to_training_examples, PageContent, PdfLoader, PdfLoaderConfig,
};
pub use tfidf_agent::TfidfAgent;

// Version and library information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Returns comprehensive version information for the library
pub fn version_info() -> String {
    format!("{} v{}", NAME, VERSION)
}

/// Provides a brief description of the library's purpose
pub fn library_description() -> String {
    "AIRust: A Modular Knowledge Intelligence Engine — find knowledge, compare text, suggest connections".to_string()
}
