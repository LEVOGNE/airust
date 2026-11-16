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

pub struct PageContent {
    pub page_number: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct PdfLoaderConfig {
    pub min_chunk_size: usize,

    pub max_chunk_size: usize,

    pub chunk_overlap: usize,

    pub default_weight: f32,

    pub include_metadata: bool,

    pub split_by_sentence: bool,

    pub split_by_paragraph: bool,

    pub clean_text: bool,

    pub remove_page_numbers: bool,

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
    pub fn new() -> Self {
        Self {
            config: PdfLoaderConfig::default(),
        }
    }

    pub fn with_config(config: PdfLoaderConfig) -> Self {
        Self { config }
    }

    pub fn pdf_to_knowledge_base<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<KnowledgeBase, PdfError> {
        let examples = self.pdf_to_training_examples(path)?;

        Ok(KnowledgeBase::from_examples(examples))
    }

    pub fn pdf_to_training_examples<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<TrainingExample>, PdfError> {
        self.config.validate()?;
        let path = path.as_ref();
        if !path.is_file() {
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

    fn is_garbage_text(text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return false;
        }

        let total = trimmed.chars().count();
        if total == 0 {
            return false;
        }

        let garbage_count = trimmed
            .chars()
            .filter(|c| {
                let cp = *c as u32;
                *c == '\u{FFFD}'
                    || (0xE000..=0xF8FF).contains(&cp)
                    || (c.is_control() && *c != '\n' && *c != '\t' && *c != '\r')
            })
            .count();

        let garbage_ratio = garbage_count as f64 / total as f64;

        let readable_count = trimmed
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_whitespace())
            .count();

        let readable_ratio = readable_count as f64 / total as f64;

        garbage_ratio > 0.15 || readable_ratio < 0.30
    }

    pub fn clean_text(&self, raw: &str) -> String {
        let mut text: String = raw.nfc().collect();

        text = text
            .replace('\u{FB00}', "ff")
            .replace('\u{FB01}', "fi")
            .replace('\u{FB02}', "fl")
            .replace('\u{FB03}', "ffi")
            .replace('\u{FB04}', "ffl");

        text = text
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t' || *c == '\r')
            .collect();

        if self.config.dehyphenate {
            text = DEHYPHEN.replace_all(&text, "$1$2").to_string();
        }

        text = text.replace('\t', " ").replace("\r\n", "\n");

        text = SPACES.replace_all(&text, " ").to_string();

        text = NEWLINES.replace_all(&text, "\n\n").to_string();

        if self.config.remove_page_numbers {
            text = text
                .lines()
                .filter(|l| !PAGE_NUMBER.is_match(l))
                .collect::<Vec<_>>()
                .join("\n");
        }

        text.trim().to_string()
    }

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

            start = end - self.config.chunk_overlap;
        }
        chunks
    }

    fn split_into_paragraph_segments(&self, text: &str) -> Vec<String> {
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut segments = Vec::new();

        for para in paragraphs {
            let trimmed = para.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.chars().count() <= self.config.max_chunk_size {
                segments.push(format!("{}\n\n", trimmed));
            } else if self.config.split_by_sentence {
                let sentences = self.split_into_sentences(trimmed);
                segments.extend(sentences);
                segments.push("\n\n".to_string());
            } else {
                segments.push(format!("{}\n\n", trimmed));
            }
        }

        segments
    }

    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        let abbreviations = [
            "Dr", "Mr", "Mrs", "Ms", "Prof", "Inc", "Ltd", "Co", "Jr", "Sr", "St", "etc", "vs",
            "ca", "Nr", "Abs", "bzw", "vgl", "sog", "Fig", "Tab", "Vol", "No", "Dept", "Gen",
            "Gov", "Sgt", "Cpl", "Pvt", "Capt", "Col", "Maj", "Rev", "Jan", "Feb", "Mar", "Apr",
            "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec", "Mon", "Tue", "Wed", "Thu", "Fri",
            "Sat", "Sun",
        ];

        let mut sentences = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            current.push(chars[i]);

            if chars[i] == '.' || chars[i] == '!' || chars[i] == '?' {
                if chars[i] == '.' && i + 2 < len && chars[i + 1] == '.' && chars[i + 2] == '.' {
                    current.push(chars[i + 1]);
                    current.push(chars[i + 2]);
                    i += 3;
                    continue;
                }

                let open_parens = current.chars().filter(|c| *c == '(').count();
                let close_parens = current.chars().filter(|c| *c == ')').count();
                if open_parens > close_parens {
                    i += 1;
                    continue;
                }

                if chars[i] == '.' {
                    let has_digit_before = i > 0 && chars[i - 1].is_ascii_digit();
                    let has_digit_after = i + 1 < len && chars[i + 1].is_ascii_digit();
                    if has_digit_before && has_digit_after {
                        i += 1;
                        continue;
                    }
                }

                if chars[i] == '.' {
                    let word_before = extract_word_before(&chars, i);
                    if abbreviations.iter().any(|a| *a == word_before) {
                        i += 1;
                        continue;
                    }

                    if !word_before.is_empty()
                        && word_before.chars().count() <= 2
                        && i + 1 < len
                        && chars[i + 1].is_alphabetic()
                    {
                        i += 1;
                        continue;
                    }
                }

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

        let trimmed = current.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
        }

        sentences
    }

    pub fn save_knowledge_base<P: AsRef<Path>>(
        &self,
        kb: &KnowledgeBase,
        path: P,
    ) -> Result<(), PdfError> {
        kb.save(Some(PathBuf::from(path.as_ref())))
            .map_err(|e| PdfError::IoError(std::io::Error::other(e.to_string())))
    }
}

fn extract_word_before(chars: &[char], pos: usize) -> String {
    if pos == 0 {
        return String::new();
    }
    let mut end = pos;

    end = end.saturating_sub(1);

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

fn skip_whitespace(chars: &[char], pos: usize) -> usize {
    let mut p = pos;
    while p < chars.len() && (chars[p] == ' ' || chars[p] == '\t') {
        p += 1;
    }
    p
}

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

        let text = "Dr. Smith went to Washington. He met Prof. Jones there.";
        let sentences = loader.split_into_sentences(text);

        assert!(
            sentences.len() == 2,
            "Expected 2 sentences, got {}: {:?}",
            sentences.len(),
            sentences
        );

        let text2 = "The value is 3.14 approximately. That is correct.";
        let sentences2 = loader.split_into_sentences(text2);
        assert!(
            sentences2.len() == 2,
            "Expected 2 sentences, got {}: {:?}",
            sentences2.len(),
            sentences2
        );

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

        let raw = "incor-\nporated";

        let result = loader.clean_text(raw);
        assert!(
            result.contains("incor-"),
            "Dehyphenation should be disabled"
        );
    }

    #[test]
    fn test_garbage_detection_binary() {
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
        let mostly_ok = "Hello world. Some text here with a \u{FFFD} or two.";
        assert!(
            !PdfLoader::is_garbage_text(mostly_ok),
            "Small amount of replacement chars is OK"
        );
    }
}
