// src/lib.rs - Updated module exports
pub mod agent;
pub mod context_agent;
pub mod knowledge;
pub mod match_agent; // New: Replaces simple_agent and fuzzy_agent
pub mod tfidf_agent;

// Re-exports for easier usage
pub use agent::{Agent, ContextualAgent, ResponseFormat, TrainableAgent, TrainingExample};
pub use context_agent::ContextAgent;
pub use knowledge::KnowledgeBase;
pub use match_agent::MatchAgent;
pub use tfidf_agent::TfidfAgent;

// Version and library information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Returns version information
pub fn version_info() -> String {
    format!("{} v{}", NAME, VERSION)
}
