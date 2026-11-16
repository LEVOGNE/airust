// src/pdf_loader.rs - PDF to Training Data Converter (pdf_oxide backend)
use crate::agent::{ResponseFormat, TrainingExample};
use crate::knowledge::KnowledgeBase;
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

static DEHYPHEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w)-\s*\n\s*(\w)").unwrap());
static SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r" {2,}").unwrap());
static NEWLINES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").unwrap());
static PAGE_NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*-?\s*\d{1,4}\s*-?\s*$").unwrap());

// Define a custom error type for PDF operations
#[derive(Debug)]
pub enum PdfError {
    IoError(std::io::Error),
    PdfError(String),
    InvalidPath(String),
    InvalidConfig(String),
}

impl fmt::Display for PdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfError::IoError(err) => write!(f, "IO Error: {}", err),
            PdfError::PdfError(msg) => write!(f, "PDF Error: {}", msg),
            PdfError::InvalidPath(path) => write!(f, "Invalid Path: {}", path),
            PdfError::InvalidConfig(message) => write!(f, "Invalid PDF configuration: {}", message),
        }
    }
}

impl Error for PdfError {}

impl From<std::io::Error> for PdfError {
    fn from(err: std::io::Error) -> Self {
        PdfError::IoError(err)
    }
}

/// Extracted content from a single PDF page
pub struct PageContent {
    pub page_number: usize,
    pub text: String,
}

/// Structure for configuring how PDFs are converted to training data
pub struct PdfLoaderConfig {
    /// Preferred minimum length (in characters); a final shorter chunk is retained
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

    /// Whether to primarily split at paragraph boundaries (\n\n)
    pub split_by_paragraph: bool,

    /// Whether to apply the text cleaning pipeline
    pub clean_text: bool,

    /// Whether to remove page number lines
    pub remove_page_numbers: bool,

    /// Whether to rejoin hyphenated words across line breaks
    pub dehyphenate: bool,
}

impl Default for PdfLoaderConfig {
    fn default() -> Self {
        Self {
            min_chunk_size: 50,
            max_chunk_size: 1000,
            chunk_overlap: 200,
            default_weight: 1.0,
            include_metadata: true,
            split_by_sentence: true,
            split_by_paragraph: true,
            clean_text: true,
            remove_page_numbers: true,
            dehyphenate: true,
        }
    }
}

impl PdfLoaderConfig {
    pub fn validate(&self) -> Result<(), PdfError> {
        if self.max_chunk_size == 0 || self.min_chunk_size > self.max_chunk_size {
            return Err(PdfError::InvalidConfig(
                "require 0 <= min_chunk_size <= max_chunk_size and max_chunk_size > 0".to_owned(),
            ));
        }
        if self.chunk_overlap >= self.max_chunk_size {
            return Err(PdfError::InvalidConfig(
                "chunk_overlap must be smaller than max_chunk_size".to_owned(),
            ));
        }
        if !self.default_weight.is_finite() || self.default_weight < 0.0 {
            return Err(PdfError::InvalidConfig(
                "default_weight must be finite and nonnegative".to_owned(),
            ));
        }
        Ok(())
    }
}

pub struct PdfLoader {
    config: PdfLoaderConfig,
}

impl Default for PdfLoader {
    fn default() -> Self {
        Self::new()
    }
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

        Ok(KnowledgeBase::from_examples(examples))
    }

    /// Loads a PDF and converts it to TrainingExamples
    pub fn pdf_to_training_examples<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<TrainingExample>, PdfError> {
        self.config.validate()?;
        let path = path.as_ref();
        if !path.exists() {
            return Err(PdfError::InvalidPath(path.to_string_lossy().to_string()));
        }

        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut examples = Vec::new();
        for page in self.extract_pages_from_pdf(path)? {
            let mut page_examples = self.text_to_training_examples(&page.text, &filename);
            for example in &mut page_examples {
                if let Some(metadata) = &mut example.metadata {
                    metadata["page_number"] = serde_json::json!(page.page_number);
                }
            }
            examples.extend(page_examples);
        }
        Ok(examples)
    }

    /// Extracts pages individually with metadata
    pub fn extract_pages_from_pdf(&self, path: &Path) -> Result<Vec<PageContent>, PdfError> {
        self.config.validate()?;
        let doc =
            pdf_oxide::PdfDocument::open(path).map_err(|e| PdfError::PdfError(format!("{}", e)))?;

        let page_count = doc
            .page_count()
            .map_err(|e| PdfError::PdfError(format!("{}", e)))?;

        let mut pages = Vec::new();

        for page_idx in 0..page_count {
            let text = doc
                .extract_text(page_idx)
                .map_err(|e| PdfError::PdfError(format!("Page {}: {}", page_idx + 1, e)))?;

            if Self::is_garbage_text(&text) {
                continue;
            }

            let cleaned = if self.config.clean_text {
                self.clean_text(&text)
            } else {
                text.trim().to_string()
            };

            if !cleaned.is_empty() {
                pages.push(PageContent {
                    page_number: page_idx + 1,
                    text: cleaned,
                });
            }
        }

        Ok(pages)
    }

    /// Detects binary garbage / non-text content from scanned or image-only pages.
    /// Returns true if the text has a high ratio of non-printable, replacement,
    /// or private-use characters. Valid Unicode letters remain readable.
    fn is_garbage_text(text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return false;
        }

        let total = trimmed.chars().count();
        if total == 0 {
            return false;
        }

        // Count characters that are "garbage" indicators:
        // - Unicode replacement char (U+FFFD)
        // - Private Use Area (U+E000..U+F8FF)
        // - Non-printable controls (excluding \n \t \r \x20)
        let garbage_count = trimmed
            .chars()
            .filter(|c| {
                let cp = *c as u32;
                *c == '\u{FFFD}'                           // replacement char
                    || (0xE000..=0xF8FF).contains(&cp)     // private use area
                    || (c.is_control() && *c != '\n' && *c != '\t' && *c != '\r')
            })
            .count();

        let garbage_ratio = garbage_count as f64 / total as f64;

        // Also check: ratio of printable ASCII + common Unicode (letters, digits, punctuation)
        let readable_count = trimmed
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_whitespace())
            .count();

        let readable_ratio = readable_count as f64 / total as f64;

        // Garbage if >15% garbage chars OR <30% readable chars
        garbage_ratio > 0.15 || readable_ratio < 0.30
    }

    /// 6-stage text cleaning pipeline
    pub fn clean_text(&self, raw: &str) -> String {
        // 1. Unicode NFC normalization
        let mut text: String = raw.nfc().collect();

        // 2. Ligature expansion
        text = text
            .replace('\u{FB00}', "ff")
            .replace('\u{FB01}', "fi")
            .replace('\u{FB02}', "fl")
            .replace('\u{FB03}', "ffi")
            .replace('\u{FB04}', "ffl");

        // 3. Remove control characters (keep \n \t \r)
        text = text
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t' || *c == '\r')
            .collect();

        // 4. Dehyphenation: word-\n -> word (rejoin hyphenated line breaks)
        if self.config.dehyphenate {
            text = DEHYPHEN.replace_all(&text, "$1$2").to_string();
        }

        // 5. Whitespace normalization
        text = text.replace('\t', " ").replace("\r\n", "\n");

        text = SPACES.replace_all(&text, " ").to_string();

        text = NEWLINES.replace_all(&text, "\n\n").to_string();

        // 6. Remove page-number-only lines
        if self.config.remove_page_numbers {
            text = text
                .lines()
                .filter(|l| !PAGE_NUMBER.is_match(l))
                .collect::<Vec<_>>()
                .join("\n");
        }

        text.trim().to_string()
    }

    /// Splits text into chunks and creates TrainingExamples with extended metadata
    fn text_to_training_examples(&self, text: &str, source_file: &str) -> Vec<TrainingExample> {
        let mut examples = Vec::new();
        let chunks = self.split_text_into_chunks(text);

        for (i, chunk) in chunks.iter().enumerate() {
            let metadata = if self.config.include_metadata {
                Some(serde_json::json!({
                    "chunk_index": i,
                    "total_chunks": chunks.len(),
                    "source_file": source_file,
                }))
            } else {
                None
            };

            examples.push(TrainingExample {
                input: chunk.clone(),
                output: ResponseFormat::Text(chunk.clone()),
                weight: self.config.default_weight,
                metadata,
            });
        }

        examples
    }

    /// Splits text into overlapping chunks while respecting UTF-8 characters
    fn split_text_into_chunks(&self, text: &str) -> Vec<String> {
        if self.config.validate().is_err() || text.trim().is_empty() {
            return Vec::new();
        }
        let text = text.trim();
        if text.chars().count() <= self.config.max_chunk_size {
            return vec![text.to_owned()];
        }
        let segments = if self.config.split_by_paragraph {
            self.split_into_paragraph_segments(text)
        } else if self.config.split_by_sentence {
            self.split_into_sentences(text)
        } else {
            vec![text.to_owned()]
        };
        let mut chars = Vec::new();
        let mut boundaries = std::collections::BTreeSet::new();
        for segment in segments {
            chars.extend(segment.chars());
            boundaries.insert(chars.len());
        }
        let mut chunks = Vec::new();
        let mut start = 0usize;
        while start < chars.len() {
            let hard_end = start
                .saturating_add(self.config.max_chunk_size)
                .min(chars.len());
            let end = if hard_end == chars.len() {
                hard_end
            } else {
                let minimum = self
                    .config
                    .min_chunk_size
                    .max(self.config.chunk_overlap + 1);
                let earliest = start.saturating_add(minimum).min(hard_end);
                boundaries
                    .range(earliest..=hard_end)
                    .next_back()
                    .copied()
                    .unwrap_or(hard_end)
            };
            let chunk: String = chars[start..end].iter().collect();
            if !chunk.trim().is_empty() {
                chunks.push(chunk.trim().to_owned());
            }
            if end == chars.len() {
                break;
            }
            // Every iteration advances, including overlap=max-1 and Unicode text.
            start = end - self.config.chunk_overlap;
        }
        chunks
    }

    /// Splits text into paragraph segments, then sub-splits large paragraphs by sentence
    fn split_into_paragraph_segments(&self, text: &str) -> Vec<String> {
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut segments = Vec::new();

        for para in paragraphs {
            let trimmed = para.trim();
            if trimmed.is_empty() {
                continue;
            }

            // If paragraph fits in a chunk, keep it as one segment
            if trimmed.chars().count() <= self.config.max_chunk_size {
                segments.push(format!("{}\n\n", trimmed));
            } else if self.config.split_by_sentence {
                // Large paragraph: sub-split by sentence
                let sentences = self.split_into_sentences(trimmed);
                segments.extend(sentences);
                segments.push("\n\n".to_string());
            } else {
                segments.push(format!("{}\n\n", trimmed));
            }
        }

        segments
    }

    /// Abbreviation-aware sentence splitting
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        let abbreviations = [
            "Dr", "Mr", "Mrs", "Ms", "Prof", "Inc", "Ltd", "Co", "Jr", "Sr", "St", "etc", "vs",
            "ca", "Nr", "Abs", "bzw", "vgl", "sog", "Fig", "Tab", "Vol", "No", "Dept", "Gen",
            "Gov", "Sgt", "Cpl", "Pvt", "Capt", "Col", "Maj", "Rev", "Jan", "Feb", "Mar", "Apr",
            "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec", "Mon", "Tue", "Wed", "Thu", "Fri",
            "Sat", "Sun",
        ];

        // Build regex: sentence-ending punctuation followed by space + uppercase or end-of-string
        // But NOT preceded by a known abbreviation
        let mut sentences = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            current.push(chars[i]);

            if chars[i] == '.' || chars[i] == '!' || chars[i] == '?' {
                // Check for ellipsis: "..."
                if chars[i] == '.' && i + 2 < len && chars[i + 1] == '.' && chars[i + 2] == '.' {
                    current.push(chars[i + 1]);
                    current.push(chars[i + 2]);
                    i += 3;
                    continue;
                }

                // Check if inside parentheses (simple heuristic)
                let open_parens = current.chars().filter(|c| *c == '(').count();
                let close_parens = current.chars().filter(|c| *c == ')').count();
                if open_parens > close_parens {
                    i += 1;
                    continue;
                }

                // Check for number context: "3.14", "100.000"
                if chars[i] == '.' {
                    let has_digit_before = i > 0 && chars[i - 1].is_ascii_digit();
                    let has_digit_after = i + 1 < len && chars[i + 1].is_ascii_digit();
                    if has_digit_before && has_digit_after {
                        i += 1;
                        continue;
                    }
                }

                // Check for abbreviation
                if chars[i] == '.' {
                    let word_before = extract_word_before(&chars, i);
                    if abbreviations.iter().any(|a| *a == word_before) {
                        i += 1;
                        continue;
                    }
                    // Single-letter abbreviation like "U.S.A." or "z.B."
                    if word_before.len() <= 2 && !word_before.is_empty() {
                        // Check if next char is also a letter (continuing abbreviation)
                        if i + 1 < len && chars[i + 1].is_alphabetic() {
                            i += 1;
                            continue;
                        }
                    }
                }

                // Now check if next non-space char is uppercase or end of text
                let next_pos = skip_whitespace(&chars, i + 1);
                if next_pos >= len || chars[next_pos].is_uppercase() || chars[next_pos] == '\n' {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        sentences.push(format!("{} ", trimmed));
                    }
                    current.clear();
                }
            }

            i += 1;
        }

        // Add the last incomplete sentence
        let trimmed = current.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
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
            .map_err(|e| PdfError::IoError(std::io::Error::other(e.to_string())))
    }
}

/// Extract the word immediately before position `pos` in the char array
fn extract_word_before(chars: &[char], pos: usize) -> String {
    if pos == 0 {
        return String::new();
    }
    let mut end = pos;
    // Skip back past the punctuation
    end = end.saturating_sub(1);
    // Collect alphabetic characters going backwards
    let mut word = String::new();
    let mut j = end;
    loop {
        if chars[j].is_alphabetic() {
            word.push(chars[j]);
        } else {
            break;
        }
        if j == 0 {
            break;
        }
        j -= 1;
    }
    word.chars().rev().collect()
}

/// Skip whitespace characters starting from `pos`
fn skip_whitespace(chars: &[char], pos: usize) -> usize {
    let mut p = pos;
    while p < chars.len() && (chars[p] == ' ' || chars[p] == '\t') {
        p += 1;
    }
    p
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
    fn long_unicode_segments_are_bounded_and_keep_the_tail() {
        for overlap in [0, 5, 19] {
            let loader = PdfLoader::with_config(PdfLoaderConfig {
                min_chunk_size: 10,
                max_chunk_size: 20,
                chunk_overlap: overlap,
                split_by_paragraph: false,
                split_by_sentence: false,
                ..Default::default()
            });
            let input = format!("{}尾", "ä😀".repeat(45));
            let chunks = loader.split_text_into_chunks(&input);
            assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 20));
            assert!(chunks.last().unwrap().ends_with('尾'));
            let mut reconstructed = chunks[0].clone();
            for chunk in chunks.iter().skip(1) {
                reconstructed.extend(chunk.chars().skip(overlap));
            }
            assert_eq!(reconstructed, input);
        }
    }

    #[test]
    fn test_clean_text_ligatures() {
        let loader = PdfLoader::new();
        let input = "e\u{FB03}cient and e\u{FB00}ective o\u{FB04}ine";
        let result = loader.clean_text(input);
        assert!(result.contains("efficient"));
        assert!(result.contains("effective"));
        assert!(result.contains("offline"));
    }

    #[test]
    fn test_clean_text_dehyphenation() {
        let loader = PdfLoader::new();
        let input = "This is incor-\nporated into the system.";
        let result = loader.clean_text(input);
        assert!(result.contains("incorporated"));
        assert!(!result.contains("incor-"));
    }

    #[test]
    fn test_clean_text_whitespace() {
        let loader = PdfLoader::new();
        let input = "Hello   world.\t\tTest.\r\nLine.\n\n\n\n\nParagraph.";
        let result = loader.clean_text(input);
        assert!(result.contains("Hello world."));
        assert!(!result.contains("   "));
        assert!(!result.contains("\t"));
        assert!(!result.contains("\r"));
        // Max 2 consecutive newlines
        assert!(!result.contains("\n\n\n"));
    }

    #[test]
    fn test_clean_text_page_numbers() {
        let loader = PdfLoader::new();
        let input = "Some text here.\n42\nMore text.\n- 7 -\nEnd.";
        let result = loader.clean_text(input);
        assert!(!result.contains("\n42\n"));
        assert!(!result.contains("- 7 -"));
        assert!(result.contains("Some text here."));
        assert!(result.contains("More text."));
    }

    #[test]
    fn test_clean_text_control_chars() {
        let loader = PdfLoader::new();
        let input = "Hello\x00\x01\x02 World\x0B\x0C test\nnewline";
        let result = loader.clean_text(input);
        assert!(result.contains("Hello World test"));
        assert!(result.contains("newline"));
    }

    #[test]
    fn test_improved_sentence_split() {
        let loader = PdfLoader::new();

        // Abbreviations should NOT cause a split
        let text = "Dr. Smith went to Washington. He met Prof. Jones there.";
        let sentences = loader.split_into_sentences(text);
        // "Dr." should not cause a split, nor "Prof."
        assert!(
            sentences.len() == 2,
            "Expected 2 sentences, got {}: {:?}",
            sentences.len(),
            sentences
        );

        // Numbers should NOT cause a split
        let text2 = "The value is 3.14 approximately. That is correct.";
        let sentences2 = loader.split_into_sentences(text2);
        assert!(
            sentences2.len() == 2,
            "Expected 2 sentences, got {}: {:?}",
            sentences2.len(),
            sentences2
        );

        // Ellipsis should NOT cause multiple splits
        let text3 = "Wait... What happened? Nothing.";
        let sentences3 = loader.split_into_sentences(text3);
        assert!(
            sentences3.len() <= 3,
            "Ellipsis caused too many splits: {:?}",
            sentences3
        );
    }

    #[test]
    fn test_paragraph_chunking() {
        let config = PdfLoaderConfig {
            min_chunk_size: 10,
            max_chunk_size: 200,
            chunk_overlap: 0,
            split_by_paragraph: true,
            split_by_sentence: true,
            clean_text: false,
            ..Default::default()
        };
        let loader = PdfLoader::with_config(config);

        let text = "First paragraph with some text.\n\nSecond paragraph with more text.\n\nThird paragraph here.";
        let chunks = loader.split_text_into_chunks(text);

        // All three paragraphs should fit in one chunk (< 200 chars)
        assert!(!chunks.is_empty(), "Should produce at least one chunk");
    }

    #[test]
    fn test_split_into_sentences_basic() {
        let loader = PdfLoader::new();
        let text = "This is a sentence. This is a second sentence! Is this a third sentence?";
        let sentences = loader.split_into_sentences(text);
        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn test_split_text_into_chunks() {
        let config = PdfLoaderConfig {
            min_chunk_size: 5,
            max_chunk_size: 20,
            chunk_overlap: 5,
            split_by_paragraph: false,
            ..Default::default()
        };
        let loader = PdfLoader::with_config(config);

        let text = "This is a test. This is another test.";
        let chunks = loader.split_text_into_chunks(text);

        assert!(chunks.len() >= 2);

        for chunk in &chunks {
            assert!(chunk.chars().count() <= 20);
        }
    }

    #[test]
    fn test_no_clean_config() {
        let config = PdfLoaderConfig {
            clean_text: false,
            dehyphenate: false,
            remove_page_numbers: false,
            ..Default::default()
        };
        let loader = PdfLoader::with_config(config);
        // With clean_text disabled, dehyphenation should still not happen
        // because clean_text gates the whole pipeline in pdf_to_training_examples
        let raw = "incor-\nporated";
        // Direct clean_text call with dehyphenate=false
        let result = loader.clean_text(raw);
        assert!(
            result.contains("incor-"),
            "Dehyphenation should be disabled"
        );
    }

    #[test]
    fn test_garbage_detection_binary() {
        // Simulated binary garbage like from image-only PDF pages
        let garbage = "v>It\u{FFFD}<\u{FFFD}o\u{FFFD}\u{FFFD}\u{FFFD}.\u{FFFD}\u{FFFD};d\u{FFFD}a\u{FFFD}P\u{FFFD}1\u{FFFD}W<\u{FFFD}\u{FFFD}\u{FFFD}R";
        assert!(
            PdfLoader::is_garbage_text(garbage),
            "Should detect binary garbage"
        );
    }

    #[test]
    fn test_garbage_detection_clean_text() {
        let clean =
            "This is a perfectly normal English sentence. It has proper words and punctuation.";
        assert!(
            !PdfLoader::is_garbage_text(clean),
            "Should NOT flag clean text as garbage"
        );
    }

    #[test]
    fn test_garbage_detection_private_use_area() {
        // Text dominated by Private Use Area characters
        let pua: String = (0..50)
            .map(|i| char::from_u32(0xE000 + i).unwrap())
            .collect();
        assert!(
            PdfLoader::is_garbage_text(&pua),
            "Should detect PUA-heavy text as garbage"
        );
    }

    #[test]
    fn test_garbage_detection_mixed() {
        // Mostly readable with a few odd chars — should pass
        let mostly_ok = "Hello world. Some text here with a \u{FFFD} or two.";
        assert!(
            !PdfLoader::is_garbage_text(mostly_ok),
            "Small amount of replacement chars is OK"
        );
    }
}
