// AIRust library module exports and version information

// Core modules
pub mod agent;
pub mod context_agent;
pub mod knowledge;
pub mod match_agent;
pub mod pdf_loader;
pub mod tfidf_agent;

// Re-exports for easier usage
pub use agent::{Agent, ContextualAgent, ResponseFormat, TrainableAgent, TrainingExample};
pub use context_agent::ContextAgent;
pub use knowledge::KnowledgeBase;
pub use match_agent::MatchAgent;
pub use pdf_loader::{pdf_to_knowledge_base, pdf_to_training_examples, PdfLoader, PdfLoaderConfig};
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
    "AIRust: A modular Rust library for building flexible AI agents with various matching strategies".to_string()
}
