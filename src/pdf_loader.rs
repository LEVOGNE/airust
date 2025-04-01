// src/pdf_loader.rs - PDF to Training Data Converter
use crate::agent::{ResponseFormat, TrainingExample};
use crate::knowledge::KnowledgeBase;
use std::error::Error;
use std::fmt;

use std::path::{Path, PathBuf};

// Define a custom error type for PDF operations
#[derive(Debug)]
pub enum PdfError {
    IoError(std::io::Error),
    PdfError(String),
    InvalidPath(String),
}

impl fmt::Display for PdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfError::IoError(err) => write!(f, "IO Error: {}", err),
            PdfError::PdfError(msg) => write!(f, "PDF Error: {}", msg),
            PdfError::InvalidPath(path) => write!(f, "Invalid Path: {}", path),
        }
    }
}

impl Error for PdfError {}

impl From<std::io::Error> for PdfError {
    fn from(err: std::io::Error) -> Self {
        PdfError::IoError(err)
    }
}

/// Structure for configuring how PDFs are converted to training data
pub struct PdfLoaderConfig {
    /// Minimum length of a chunk (in characters)
    pub min_chunk_size: usize,

    /// Maximum length of a chunk (in characters)
    pub max_chunk_size: usize,

    /// Overlap between chunks (in characters)
    pub chunk_overlap: usize,

    /// Default weight for generated training examples
    pub default_weight: f32,

    /// Whether metadata like page number and position in the document should be added
    pub include_metadata: bool,

    /// Whether chunks should be split at sentence boundaries
    pub split_by_sentence: bool,
}

impl Default for PdfLoaderConfig {
    fn default() -> Self {
        Self {
            min_chunk_size: 50,      // At least 50 characters per chunk
            max_chunk_size: 1000,    // Maximum 1000 characters per chunk
            chunk_overlap: 200,      // 200 characters overlap
            default_weight: 1.0,     // Default weight for all chunks
            include_metadata: true,  // Include metadata by default
            split_by_sentence: true, // Split at sentence boundaries
        }
    }
}

pub struct PdfLoader {
    config: PdfLoaderConfig,
}

impl PdfLoader {
    /// Creates a new PDF loader with default configuration
    pub fn new() -> Self {
        Self {
            config: PdfLoaderConfig::default(),
        }
    }

    /// Creates a new PDF loader with custom configuration
    pub fn with_config(config: PdfLoaderConfig) -> Self {
        Self { config }
    }

    /// Loads a PDF and converts it to a KnowledgeBase
    pub fn pdf_to_knowledge_base<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<KnowledgeBase, PdfError> {
        let examples = self.pdf_to_training_examples(path)?;

        let mut kb = KnowledgeBase::new();
        for example in examples {
            kb.add_example(example.input, example.output, example.weight);
        }

        Ok(kb)
    }

    /// Loads a PDF and converts it to TrainingExamples
    pub fn pdf_to_training_examples<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<TrainingExample>, PdfError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(PdfError::InvalidPath(path.to_string_lossy().to_string()));
        }

        // Extract text from PDF
        let text = self.extract_text_from_pdf(path)?;

        // Split text into chunks and convert to TrainingExamples
        let examples = self.text_to_training_examples(&text);

        Ok(examples)
    }

    /// Extracts text from a PDF file
    fn extract_text_from_pdf(&self, path: &Path) -> Result<String, PdfError> {
        // Use pdf-extract to extract text
        match pdf_extract::extract_text(path) {
            Ok(text) => Ok(text),
            Err(e) => Err(PdfError::PdfError(format!("Error extracting text: {}", e))),
        }
    }

    /// Splits text into chunks and creates TrainingExamples
    fn text_to_training_examples(&self, text: &str) -> Vec<TrainingExample> {
        let mut examples = Vec::new();
        let chunks = self.split_text_into_chunks(text);

        for (i, chunk) in chunks.iter().enumerate() {
            // Create metadata if configured
            let metadata = if self.config.include_metadata {
                Some(serde_json::json!({
                    "chunk_index": i,
                    "total_chunks": chunks.len(),
                }))
            } else {
                None
            };

            // Create TrainingExample
            examples.push(TrainingExample {
                input: chunk.clone(),                        // The text chunk is the input
                output: ResponseFormat::Text(chunk.clone()), // The same text as output
                weight: self.config.default_weight,
                metadata,
            });
        }

        examples
    }

    /// Splits text into overlapping chunks while respecting UTF-8 characters
    fn split_text_into_chunks(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let text = text.trim();

        if text.is_empty() {
            return chunks;
        }

        // If the text is shorter than the maximum chunk size, return it as a single chunk
        if text.chars().count() <= self.config.max_chunk_size {
            chunks.push(text.to_string());
            return chunks;
        }

        // Split text into sentences if configured
        let segments = if self.config.split_by_sentence {
            self.split_into_sentences(text)
        } else {
            // Otherwise split into characters - but correctly as UTF-8
            text.chars().map(|c| c.to_string()).collect()
        };

        let mut current_chunk = String::new();

        for segment in segments {
            // If adding the segment would exceed max_chunk_size
            if current_chunk.chars().count() + segment.chars().count() > self.config.max_chunk_size
            {
                // If the current chunk is large enough, save it
                if current_chunk.chars().count() >= self.config.min_chunk_size {
                    chunks.push(current_chunk.clone());

                    // Start new chunk with overlap
                    if self.config.chunk_overlap > 0 {
                        // Keep the last chunk_overlap characters (UTF-8 safe)
                        if current_chunk.chars().count() > self.config.chunk_overlap {
                            let chars: Vec<char> = current_chunk.chars().collect();
                            let overlap_start = chars.len() - self.config.chunk_overlap;
                            current_chunk = chars[overlap_start..].iter().collect();
                        }
                    } else {
                        current_chunk.clear();
                    }
                }
            }

            // Add segment to current chunk
            current_chunk.push_str(&segment);

            // If the chunk is now larger than max_chunk_size, split it
            let chunk_char_count = current_chunk.chars().count();
            if chunk_char_count > self.config.max_chunk_size {
                // Safe splitting considering UTF-8 characters
                let chars: Vec<char> = current_chunk.chars().collect();
                chunks.push(chars[..self.config.max_chunk_size].iter().collect());

                // Keep remainder with overlap
                if self.config.chunk_overlap > 0 {
                    let overlap_start = self.config.max_chunk_size - self.config.chunk_overlap;
                    current_chunk = chars[overlap_start..].iter().collect();
                } else {
                    current_chunk = chars[self.config.max_chunk_size..].iter().collect();
                }
            }
        }

        // Add the last chunk if it's large enough
        if !current_chunk.is_empty() && current_chunk.chars().count() >= self.config.min_chunk_size
        {
            chunks.push(current_chunk);
        }

        chunks
    }

    /// Splits text into sentences
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        // Simple sentence splitting based on periods, question marks, and exclamation marks
        // Can be replaced with a more complex NLP solution
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();

        for c in text.chars() {
            current_sentence.push(c);

            // Check if the end of a sentence has been reached
            if ['.', '!', '?'].contains(&c) {
                sentences.push(current_sentence.clone());
                current_sentence.clear();
            }
        }

        // Add the last incomplete sentence if any
        if !current_sentence.is_empty() {
            sentences.push(current_sentence);
        }

        sentences
    }

    /// Saves a KnowledgeBase to a file
    pub fn save_knowledge_base<P: AsRef<Path>>(
        &self,
        kb: &KnowledgeBase,
        path: P,
    ) -> Result<(), PdfError> {
        kb.save(Some(PathBuf::from(path.as_ref())))
            .map_err(|e| PdfError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }
}

// Helper functions for easier usage
pub fn pdf_to_knowledge_base<P: AsRef<Path>>(path: P) -> Result<KnowledgeBase, PdfError> {
    PdfLoader::new().pdf_to_knowledge_base(path)
}

pub fn pdf_to_training_examples<P: AsRef<Path>>(path: P) -> Result<Vec<TrainingExample>, PdfError> {
    PdfLoader::new().pdf_to_training_examples(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_sentences() {
        let loader = PdfLoader::new();
        let text = "This is a sentence. This is a second sentence! Is this a third sentence?";
        let sentences = loader.split_into_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "This is a sentence.");
        assert_eq!(sentences[1], "This is a second sentence!");
        assert_eq!(sentences[2], "Is this a third sentence?");
    }

    #[test]
    fn test_split_text_into_chunks() {
        let config = PdfLoaderConfig {
            min_chunk_size: 5,
            max_chunk_size: 20,
            chunk_overlap: 5,
            ..Default::default()
        };
        let loader = PdfLoader::with_config(config);

        // A short text that should fit into 2 chunks
        let text = "This is a test. This is another test.";
        let chunks = loader.split_text_into_chunks(text);

        // Check if we have at least 2 chunks
        assert!(chunks.len() >= 2);

        // Check if each chunk doesn't exceed the maximum size
        for chunk in &chunks {
            assert!(chunk.len() <= 20);
        }

        // Check if there are overlaps
        if chunks.len() >= 2 {
            let overlap = chunks[0].chars().rev().take(5).collect::<String>();

            let start_of_second = chunks[1].chars().take(overlap.len()).collect::<String>();

            assert_eq!(overlap, start_of_second.chars().rev().collect::<String>());
        }
    }
}
