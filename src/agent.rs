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
            ResponseFormat::Json(json) => write!(f, "{}", json.to_string()),
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
    pub output: ResponseFormat,

    /// Gewichtung des Beispiels (höhere Werte bedeuten höhere Priorität)
    #[serde(default = "default_weight")]
    pub weight: f32,

    /// Optionale Metadaten für das Beispiel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
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
            ResponseFormat::Text(ref s) | ResponseFormat::Markdown(ref s) => {
                if s.contains("No matching answer found")
                    || s.contains("No training data available")
                {
                    0.0
                } else {
                    1.0
                }
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
    /// Trainiert den Agenten mit einer Liste von Beispielen
    fn train(&mut self, data: &[TrainingExample]);

    /// Trainiert mit einem einzelnen Beispiel
    fn train_single(&mut self, example: &TrainingExample) {
        self.train(&[example.clone()]);
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

    /// Fügt ein neues Trainingsbeispiel hinzu und trainiert den Agenten
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
    use unicode_normalization::UnicodeNormalization;

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
        text.to_lowercase()
            .chars()
            .filter(|&c| c.is_alphabetic() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
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
        if a.is_empty() {
            return b.chars().count();
        }
        if b.is_empty() {
            return a.chars().count();
        }

        let a_chars: Vec<char> = a.chars().collect();
        let _a_len = a_chars.len(); // Markiere als intentional ungenutzt
        let b_chars: Vec<char> = b.chars().collect();
        let b_len = b_chars.len();

        let mut cache: Vec<usize> = (0..=b_len).collect();
        let mut _distance: usize; // Markiere als optional
        let mut distances = vec![0; b_len + 1];

        for (i, a_char) in a_chars.iter().enumerate() {
            distances[0] = i + 1;
            let _previous_distance = distances[0]; // Entferne unnötige Zuweisung

            for (j, b_char) in b_chars.iter().enumerate() {
                let cost = if a_char == b_char { 0 } else { 1 };
                distances[j + 1] = std::cmp::min(
                    std::cmp::min(distances[j] + 1, cache[j + 1] + 1),
                    cache[j] + cost,
                );
            }

            std::mem::swap(&mut cache, &mut distances);
        }

        cache[b_len]
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
        text.to_lowercase()
            .nfkd()
            .collect::<String>()
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
