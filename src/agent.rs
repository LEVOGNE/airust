// src/agent.rs - Erweiterte Trait-Hierarchie und Basistypen
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Fehlertypen für Agent-Operationen
#[derive(Error, Debug)]
pub enum AgentError {
    /// Keine passende Antwort in der Wissensbasis gefunden
    #[error("Keine passende Antwort gefunden")]
    NoMatchError,

    /// Keine Trainingsdaten vorhanden
    #[error("Keine Trainingsdaten verfügbar")]
    NoTrainingDataError,

    /// Trainingsfehler
    #[error("Trainingsfehler: {0}")]
    TrainingError(String),

    /// Ungültige Eingabe
    #[error("Ungültige Eingabe: {0}")]
    InvalidInputError(String),

    /// Interner Fehler
    #[error("Interner Fehler: {0}")]
    InternalError(String),

    /// I/O-Fehler beim Lesen oder Schreiben von Dateien
    #[error("I/O-Fehler: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialisierungs-/Deserialisierungsfehler
    #[error("Serialisierungsfehler: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Index außerhalb des gültigen Bereichs
    #[error("Index {0} außerhalb des gültigen Bereichs")]
    IndexOutOfBounds(usize),
}

/// Repräsentiert die möglichen Antwortformate eines Agenten
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ResponseFormat {
    /// Einfacher Textstring ohne Formatierung
    Text(String),

    /// Text im Markdown-Format mit Unterstützung für Formatierung
    Markdown(String),

    /// Strukturierte Daten im JSON-Format
    Json(serde_json::Value),
}

impl Default for ResponseFormat {
    fn default() -> Self {
        ResponseFormat::Text(String::new())
    }
}

impl fmt::Display for ResponseFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResponseFormat::Text(text) => write!(f, "{}", text),
            ResponseFormat::Markdown(md) => write!(f, "{}", md),
            ResponseFormat::Json(json) => write!(f, "{}", json),
        }
    }
}

// Konvertierung von ResponseFormat zu String für Rückwärtskompatibilität
impl From<ResponseFormat> for String {
    fn from(format: ResponseFormat) -> Self {
        match format {
            ResponseFormat::Text(text) => text,
            ResponseFormat::Markdown(md) => md,
            ResponseFormat::Json(json) => json.to_string(),
        }
    }
}

impl From<String> for ResponseFormat {
    fn from(text: String) -> Self {
        ResponseFormat::Text(text)
    }
}

impl From<&str> for ResponseFormat {
    fn from(text: &str) -> Self {
        ResponseFormat::Text(text.to_string())
    }
}

/// Training Example - Die Grundeinheit für das Training von Agenten
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TrainingExample {
    /// Die Eingabe (z.B. eine Frage oder ein Prompt)
    pub input: String,

    /// Die erwartete Ausgabe
    #[serde(deserialize_with = "deserialize_response")]
    pub output: ResponseFormat,

    /// Gewichtung des Beispiels (höhere Werte bedeuten höhere Priorität)
    #[serde(default = "default_weight")]
    pub weight: f32,

    /// Optionale Metadaten für das Beispiel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

fn deserialize_response<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<ResponseFormat, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum CompatibleResponse {
        Structured(ResponseFormat),
        Legacy(String),
    }
    Ok(match CompatibleResponse::deserialize(deserializer)? {
        CompatibleResponse::Structured(response) => response,
        CompatibleResponse::Legacy(text) => ResponseFormat::Text(text),
    })
}

/// Für die Rückwärtskompatibilität mit älteren Versionen
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct LegacyTrainingExample {
    pub input: String,
    pub output: String,
    #[serde(default = "default_weight")]
    pub weight: f32,
}

impl From<LegacyTrainingExample> for TrainingExample {
    fn from(legacy: LegacyTrainingExample) -> Self {
        Self {
            input: legacy.input,
            output: ResponseFormat::Text(legacy.output),
            weight: legacy.weight,
            metadata: None,
        }
    }
}

/// Standardgewicht für Trainingsbeispiele
pub fn default_weight() -> f32 {
    1.0
}

/// Ergebnis einer Vorhersage mit zusätzlichen Metadaten
#[derive(Debug, Clone)]
pub struct PredictionResult {
    /// Die vorhergesagte Antwort
    pub response: ResponseFormat,

    /// Konfidenz der Vorhersage (0.0 - 1.0)
    pub confidence: f32,

    /// Optionale Metadaten zur Vorhersage
    pub metadata: Option<serde_json::Value>,
}

impl From<ResponseFormat> for PredictionResult {
    fn from(response: ResponseFormat) -> Self {
        Self {
            response,
            confidence: 1.0,
            metadata: None,
        }
    }
}

impl From<PredictionResult> for ResponseFormat {
    fn from(result: PredictionResult) -> Self {
        result.response
    }
}

/// Haupttrait für alle Agenten - definiert die grundlegende Funktionalität
pub trait Agent {
    /// Verarbeitet eine Eingabe und gibt eine passende Antwort zurück
    fn predict(&self, input: &str) -> ResponseFormat;

    /// Erweiterte Vorhersage mit Metadaten und Konfidenz
    fn predict_with_metadata(&self, input: &str) -> PredictionResult {
        PredictionResult {
            response: self.predict(input),
            confidence: self.confidence(input),
            metadata: None,
        }
    }

    /// Bestimmt die Konfidenz des Agenten für eine bestimmte Eingabe (0.0 - 1.0)
    fn confidence(&self, input: &str) -> f32 {
        let response = self.predict(input);
        match response {
            ResponseFormat::Text(ref s) | ResponseFormat::Markdown(ref s)
                if s.contains("No matching answer found")
                    || s.contains("No training data available") =>
            {
                0.0
            }
            _ => 1.0,
        }
    }

    /// Prüft, ob der Agent die Eingabe beantworten kann
    fn can_answer(&self, input: &str) -> bool {
        self.confidence(input) > 0.5
    }

    /// Hilfsmethode für Rückwärtskompatibilität
    fn predict_text(&self, input: &str) -> String {
        self.predict(input).into()
    }
}

/// Trait für Agenten, die mit Beispielen trainiert werden können
pub trait TrainableAgent: Agent {
    /// Trainiert den Agenten mit einer Liste von Beispielen (ersetzt alle vorherigen Daten)
    fn train(&mut self, data: &[TrainingExample]);

    /// Fügt Beispiele hinzu ohne vorherige Daten zu ersetzen
    fn append(&mut self, data: &[TrainingExample]);

    /// Fügt ein einzelnes Beispiel hinzu (ohne vorherige Daten zu ersetzen)
    fn train_single(&mut self, example: &TrainingExample) {
        self.append(std::slice::from_ref(example));
    }

    /// Hilfsmethode für das Training mit Legacy-Daten
    fn train_legacy(&mut self, data: &[LegacyTrainingExample]) {
        let converted: Vec<TrainingExample> = data
            .iter()
            .map(|ex| TrainingExample {
                input: ex.input.clone(),
                output: ResponseFormat::Text(ex.output.clone()),
                weight: ex.weight,
                metadata: None,
            })
            .collect();

        self.train(&converted);
    }

    /// Fügt ein neues Trainingsbeispiel hinzu (ohne vorherige Daten zu ersetzen)
    fn add_example(&mut self, input: &str, output: impl Into<ResponseFormat>, weight: f32) {
        let example = TrainingExample {
            input: input.to_string(),
            output: output.into(),
            weight,
            metadata: None,
        };
        self.train_single(&example);
    }
}

/// Trait für Agenten, die Kontextinformationen nutzen können
pub trait ContextualAgent: Agent {
    /// Fügt eine Frage-Antwort-Paar zum Kontext hinzu
    fn add_context(&mut self, question: String, answer: ResponseFormat);

    /// Hilfsmethode für Textantworten
    fn add_text_context(&mut self, question: String, answer: String) {
        self.add_context(question, ResponseFormat::Text(answer));
    }

    /// Leert den Kontext
    fn clear_context(&mut self);
}

/// Trait für Agenten, die Konfidenzwerte für ihre Vorhersagen bereitstellen
pub trait ConfidenceAgent: Agent {
    /// Berechnet einen detaillierten Konfidenzwert für eine Eingabe
    fn calculate_confidence(&self, input: &str) -> f32;

    /// Gibt mehrere Antworten mit Konfidenzwerten zurück
    fn predict_top_n(&self, input: &str, n: usize) -> Vec<PredictionResult>;
}

/// Allgemeine Textverarbeitungsfunktionen
pub mod text_utils {
    use once_cell::sync::Lazy;
    use regex::Regex;
    use std::collections::HashSet;

    /// Regulärer Ausdruck zur Identifizierung von Wortzeichen
    pub static WORD_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^\p{L}\p{N}]").unwrap());

    /// Stopwörter für verschiedene Sprachen
    pub static STOPWORDS_DE: Lazy<HashSet<&'static str>> = Lazy::new(|| {
        [
            "der", "die", "das", "und", "in", "ist", "von", "mit", "zum", "zur", "zu", "ein",
            "eine", "eines",
        ]
        .iter()
        .copied()
        .collect()
    });

    pub static STOPWORDS_EN: Lazy<HashSet<&'static str>> = Lazy::new(|| {
        [
            "the", "and", "is", "in", "of", "to", "a", "with", "for", "on", "at", "this", "that",
        ]
        .iter()
        .copied()
        .collect()
    });

    /// Tokenisiert Text in einzelne Wörter
    pub fn tokenize(text: &str) -> Vec<String> {
        crate::text::tokenize(text)
    }

    /// Findet eindeutige Begriffe in einem Text
    pub fn unique_terms(text: &str) -> HashSet<String> {
        tokenize(text).into_iter().collect()
    }

    /// Entfernt Stoppwörter aus einer Liste von Tokens
    pub fn remove_stopwords(tokens: Vec<String>, lang: &str) -> Vec<String> {
        let stopwords = match lang.to_lowercase().as_str() {
            "de" | "deu" | "german" => &STOPWORDS_DE,
            _ => &STOPWORDS_EN, // Standardmäßig Englisch
        };

        tokens
            .into_iter()
            .filter(|token| !stopwords.contains(token.as_str()))
            .collect()
    }

    /// Berechnet Levenshtein-Distanz zwischen zwei Strings
    pub fn levenshtein_distance(a: &str, b: &str) -> usize {
        crate::matching::levenshtein_distance(a, b)
    }

    /// Berechnet die Jaccard-Ähnlichkeit zwischen zwei Strings
    pub fn jaccard_similarity(a: &str, b: &str) -> f32 {
        let set_a: HashSet<_> = tokenize(a).into_iter().collect();
        let set_b: HashSet<_> = tokenize(b).into_iter().collect();

        let intersection = set_a.intersection(&set_b).count() as f32;
        let union = set_a.union(&set_b).count() as f32;

        if union == 0.0 {
            0.0
        } else {
            intersection / union
        }
    }

    /// Erstellt N-Gramme aus einem Text
    pub fn create_ngrams(text: &str, n: usize) -> Vec<String> {
        if text.is_empty() || n == 0 {
            return Vec::new();
        }

        let chars: Vec<char> = text.chars().collect();
        if chars.len() < n {
            return vec![text.to_string()];
        }

        (0..=chars.len() - n)
            .map(|i| chars[i..i + n].iter().collect::<String>())
            .collect()
    }

    /// Normalisiert Text für verschiedene Verarbeitungsschritte
    pub fn normalize_text(text: &str) -> String {
        crate::text::normalize(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === ResponseFormat Tests ===

    #[test]
    fn test_response_format_conversion() {
        let text = ResponseFormat::Text("Hello".to_string());
        let string: String = text.clone().into();
        assert_eq!(string, "Hello");

        let json = ResponseFormat::Json(serde_json::json!({"key": "value"}));
        let string: String = json.into();
        assert_eq!(string, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_response_format_display_text() {
        let text = ResponseFormat::Text("Hello World".to_string());
        assert_eq!(format!("{}", text), "Hello World");
    }

    #[test]
    fn test_response_format_display_markdown() {
        let md = ResponseFormat::Markdown("# Title".to_string());
        assert_eq!(format!("{}", md), "# Title");
    }

    #[test]
    fn test_response_format_display_json() {
        let json = ResponseFormat::Json(serde_json::json!({"a": 1}));
        assert_eq!(format!("{}", json), r#"{"a":1}"#);
    }

    #[test]
    fn test_response_format_default() {
        let default = ResponseFormat::default();
        assert_eq!(String::from(default), "");
    }

    #[test]
    fn test_response_format_from_string() {
        let rf: ResponseFormat = "hello".to_string().into();
        assert_eq!(String::from(rf), "hello");
    }

    #[test]
    fn test_response_format_from_str() {
        let rf: ResponseFormat = "world".into();
        assert_eq!(String::from(rf), "world");
    }

    #[test]
    fn test_response_format_markdown_into_string() {
        let md = ResponseFormat::Markdown("**bold**".to_string());
        let s: String = md.into();
        assert_eq!(s, "**bold**");
    }

    // === LegacyTrainingExample Tests ===

    #[test]
    fn test_legacy_conversion() {
        let legacy = LegacyTrainingExample {
            input: "hello".to_string(),
            output: "world".to_string(),
            weight: 2.0,
        };

        let modern: TrainingExample = legacy.into();
        assert_eq!(modern.input, "hello");
        assert_eq!(String::from(modern.output), "world");
        assert_eq!(modern.weight, 2.0);
    }

    #[test]
    fn test_legacy_conversion_default_weight() {
        let legacy = LegacyTrainingExample {
            input: "q".to_string(),
            output: "a".to_string(),
            weight: default_weight(),
        };
        let modern: TrainingExample = legacy.into();
        assert_eq!(modern.weight, 1.0);
        assert!(modern.metadata.is_none());
    }

    // === PredictionResult Tests ===

    #[test]
    fn test_prediction_result_from_response_format() {
        let rf = ResponseFormat::Text("answer".to_string());
        let pr: PredictionResult = rf.into();
        assert_eq!(pr.confidence, 1.0);
        assert!(pr.metadata.is_none());
        assert_eq!(String::from(pr.response), "answer");
    }

    #[test]
    fn test_prediction_result_into_response_format() {
        let pr = PredictionResult {
            response: ResponseFormat::Text("result".to_string()),
            confidence: 0.8,
            metadata: None,
        };
        let rf: ResponseFormat = pr.into();
        assert_eq!(String::from(rf), "result");
    }

    // === Tokenize Tests ===

    #[test]
    fn test_text_utils() {
        let tokens = text_utils::tokenize("Hello, world! How are you?");
        assert_eq!(tokens, vec!["hello", "world", "how", "are", "you"]);

        let unique = text_utils::unique_terms("hello hello world");
        assert_eq!(unique.len(), 2);
        assert!(unique.contains("hello"));
        assert!(unique.contains("world"));

        let distance = text_utils::levenshtein_distance("kitten", "sitting");
        assert_eq!(distance, 3);

        let similarity = text_utils::jaccard_similarity("hello world", "world hello");
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_tokenize_empty() {
        assert!(text_utils::tokenize("").is_empty());
    }

    #[test]
    fn test_tokenize_only_punctuation() {
        assert!(text_utils::tokenize("!@#$%^&*()").is_empty());
    }

    #[test]
    fn test_tokenize_unicode() {
        let tokens = text_utils::tokenize("café über straße");
        assert_eq!(tokens, vec!["café", "über", "straße"]);
    }

    #[test]
    fn test_tokenize_numbers_preserved() {
        let tokens = text_utils::tokenize("hello 123 world");
        assert_eq!(tokens, vec!["hello", "123", "world"]);
    }

    #[test]
    fn test_tokenize_extra_whitespace() {
        let tokens = text_utils::tokenize("  hello   world  ");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    // === unique_terms Tests ===

    #[test]
    fn test_unique_terms_empty() {
        assert!(text_utils::unique_terms("").is_empty());
    }

    #[test]
    fn test_unique_terms_all_same() {
        let unique = text_utils::unique_terms("hello hello hello");
        assert_eq!(unique.len(), 1);
        assert!(unique.contains("hello"));
    }

    // === remove_stopwords Tests ===

    #[test]
    fn test_remove_stopwords_english() {
        let tokens = vec!["the", "cat", "is", "on", "a", "mat"]
            .into_iter()
            .map(String::from)
            .collect();
        let filtered = text_utils::remove_stopwords(tokens, "en");
        assert_eq!(filtered, vec!["cat", "mat"]);
    }

    #[test]
    fn test_remove_stopwords_german() {
        let tokens = vec!["der", "hund", "ist", "in", "dem", "haus"]
            .into_iter()
            .map(String::from)
            .collect();
        let filtered = text_utils::remove_stopwords(tokens, "de");
        assert_eq!(filtered, vec!["hund", "dem", "haus"]);
    }

    #[test]
    fn test_remove_stopwords_german_variants() {
        let tokens = vec!["hund".to_string()];
        // "deu" and "german" should also use German stopwords
        let filtered_deu = text_utils::remove_stopwords(tokens.clone(), "deu");
        assert_eq!(filtered_deu, vec!["hund"]);
        let filtered_german = text_utils::remove_stopwords(tokens, "german");
        assert_eq!(filtered_german, vec!["hund"]);
    }

    #[test]
    fn test_remove_stopwords_unknown_lang_defaults_to_english() {
        let tokens = vec!["the", "cat"].into_iter().map(String::from).collect();
        let filtered = text_utils::remove_stopwords(tokens, "fr");
        assert_eq!(filtered, vec!["cat"]);
    }

    #[test]
    fn test_remove_stopwords_all_stopwords() {
        let tokens = vec!["the", "and", "is", "in"]
            .into_iter()
            .map(String::from)
            .collect();
        let filtered = text_utils::remove_stopwords(tokens, "en");
        assert!(filtered.is_empty());
    }

    // === levenshtein_distance Tests ===

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(text_utils::levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_empty_both() {
        assert_eq!(text_utils::levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_levenshtein_one_empty() {
        assert_eq!(text_utils::levenshtein_distance("", "abc"), 3);
        assert_eq!(text_utils::levenshtein_distance("abc", ""), 3);
    }

    #[test]
    fn test_levenshtein_symmetric() {
        let d1 = text_utils::levenshtein_distance("abc", "xyz");
        let d2 = text_utils::levenshtein_distance("xyz", "abc");
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_levenshtein_single_insertion() {
        assert_eq!(text_utils::levenshtein_distance("cat", "cats"), 1);
    }

    #[test]
    fn test_levenshtein_single_deletion() {
        assert_eq!(text_utils::levenshtein_distance("cats", "cat"), 1);
    }

    #[test]
    fn test_levenshtein_unicode() {
        assert_eq!(text_utils::levenshtein_distance("über", "uber"), 1);
    }

    // === jaccard_similarity Tests ===

    #[test]
    fn test_jaccard_identical() {
        assert_eq!(
            text_utils::jaccard_similarity("hello world", "hello world"),
            1.0
        );
    }

    #[test]
    fn test_jaccard_disjoint() {
        assert_eq!(text_utils::jaccard_similarity("hello", "world"), 0.0);
    }

    #[test]
    fn test_jaccard_partial() {
        let sim = text_utils::jaccard_similarity("hello world", "hello");
        assert!(sim > 0.0 && sim < 1.0);
        // intersection = {hello}, union = {hello, world} => 1/2
        assert!((sim - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_jaccard_empty() {
        assert_eq!(text_utils::jaccard_similarity("", ""), 0.0);
    }

    #[test]
    fn test_jaccard_order_independent() {
        let s1 = text_utils::jaccard_similarity("hello world", "world hello");
        let s2 = text_utils::jaccard_similarity("world hello", "hello world");
        assert_eq!(s1, s2);
        assert_eq!(s1, 1.0);
    }

    // === create_ngrams Tests ===

    #[test]
    fn test_ngrams_basic() {
        let ngrams = text_utils::create_ngrams("hello", 2);
        assert_eq!(ngrams, vec!["he", "el", "ll", "lo"]);
    }

    #[test]
    fn test_ngrams_n_greater_than_len() {
        let ngrams = text_utils::create_ngrams("hi", 5);
        assert_eq!(ngrams, vec!["hi"]);
    }

    #[test]
    fn test_ngrams_n_zero() {
        assert!(text_utils::create_ngrams("hello", 0).is_empty());
    }

    #[test]
    fn test_ngrams_empty_text() {
        assert!(text_utils::create_ngrams("", 2).is_empty());
    }

    #[test]
    fn test_ngrams_unicode() {
        let ngrams = text_utils::create_ngrams("über", 2);
        assert_eq!(ngrams, vec!["üb", "be", "er"]);
    }

    #[test]
    fn test_ngrams_n_equals_len() {
        let ngrams = text_utils::create_ngrams("abc", 3);
        assert_eq!(ngrams, vec!["abc"]);
    }

    // === normalize_text Tests ===

    #[test]
    fn test_normalize_text_lowercase() {
        assert_eq!(text_utils::normalize_text("HELLO"), "hello");
    }

    #[test]
    fn test_normalize_text_trim() {
        assert_eq!(text_utils::normalize_text("  hello  "), "hello");
    }

    #[test]
    fn test_normalize_text_nfkd() {
        // NFKD decomposes characters, e.g. "ﬁ" (U+FB01) -> "fi"
        let normalized = text_utils::normalize_text("ﬁle");
        assert_eq!(normalized, "file");
    }
}
