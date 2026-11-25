use crate::agent::{AgentError, ResponseFormat, TrainingExample};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl KnowledgeEntry {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationKind {
    Related,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationSuggestion {
    pub source_id: String,
    pub target_id: String,
    pub kind: RelationKind,

    pub score: f64,
    pub shared_terms: Vec<String>,
}

/// Training examples with cheap read-only snapshots. Clones retain their own
/// values; the first mutation of shared examples copies the complete collection.
#[derive(Clone)]
pub struct KnowledgeBase {
    examples: Arc<Vec<TrainingExample>>,
    file_path: Option<PathBuf>,
}

pub static EMBEDDED_DATA: Lazy<Arc<Vec<TrainingExample>>> = Lazy::new(|| {
    let raw = include_str!(concat!(env!("OUT_DIR"), "/train.json"));

    match serde_json::from_str::<Vec<TrainingExample>>(raw).map_err(crate::input::json_error) {
        Ok(examples) => Arc::new(examples),
        Err(e) => {
            eprintln!("Error loading embedded training data: {}", e);
            Arc::new(Vec::new())
        }
    }
});

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            examples: Arc::new(Vec::new()),
            file_path: None,
        }
    }

    pub fn from_embedded() -> Self {
        Self {
            examples: Arc::clone(&EMBEDDED_DATA),
            file_path: None,
        }
    }

    /// Loads a trusted JSON file without a byte cap. Prefer `load_bounded` for
    /// caller-supplied paths. Validates all examples before returning a new base.
    pub fn load(path: PathBuf) -> Result<Self, AgentError> {
        let data = fs::read_to_string(&path)?;

        let examples: Vec<TrainingExample> =
            serde_json::from_str(&data).map_err(crate::input::json_error)?;
        for example in &examples {
            example.validate()?;
        }
        Ok(Self {
            examples: Arc::new(examples),
            file_path: Some(path),
        })
    }

    /// Loads a regular JSON file of at most `max_bytes` bytes, validating every weight.
    ///
    /// This bounds file bytes, not the total memory needed for parsed examples.
    /// The path must be application-owned; concurrent hostile path replacement is
    /// outside this API's contract. Use `load` only for trusted, prebounded files.
    pub fn load_bounded(path: PathBuf, max_bytes: usize) -> Result<Self, AgentError> {
        let data = crate::input::BoundedFile::open(&path, max_bytes as u64)
            .and_then(|file| file.read())
            .map_err(crate::input::InputError::into_io)?;
        let text = std::str::from_utf8(&data)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let examples: Vec<TrainingExample> =
            serde_json::from_str(text).map_err(crate::input::json_error)?;
        for example in &examples {
            example.validate()?;
        }
        Ok(Self {
            examples: Arc::new(examples),
            file_path: Some(path),
        })
    }

    pub fn save(&self, path: Option<PathBuf>) -> Result<(), AgentError> {
        let path = path
            .or_else(|| self.file_path.clone())
            .ok_or_else(|| AgentError::InvalidInputError("No path provided".to_string()))?;

        for example in self.examples.iter() {
            example.validate()?;
        }
        let json = serde_json::to_string_pretty(self.examples.as_ref())?;
        crate::persistence::atomic_write(&path, json.as_bytes())?;

        Ok(())
    }

    pub fn add_example(&mut self, input: String, output: impl Into<ResponseFormat>, weight: f32) {
        let example = TrainingExample {
            input,
            output: output.into(),
            weight,
            metadata: None,
        };
        Arc::make_mut(&mut self.examples).push(example);
    }

    pub fn remove_example(&mut self, index: usize) -> Result<TrainingExample, AgentError> {
        if index < self.examples.len() {
            Ok(Arc::make_mut(&mut self.examples).remove(index))
        } else {
            Err(AgentError::IndexOutOfBounds(index))
        }
    }

    pub fn get_examples(&self) -> &[TrainingExample] {
        &self.examples
    }

    pub fn from_examples(examples: Vec<TrainingExample>) -> Self {
        Self {
            examples: Arc::new(examples),
            file_path: None,
        }
    }

    pub fn push_example(&mut self, example: TrainingExample) {
        Arc::make_mut(&mut self.examples).push(example);
    }

    pub fn merge(&mut self, other: &KnowledgeBase) {
        if !other.examples.is_empty() {
            Arc::make_mut(&mut self.examples).extend_from_slice(&other.examples);
        }
    }

    pub fn merge_embedded(&mut self) {
        if !EMBEDDED_DATA.is_empty() {
            Arc::make_mut(&mut self.examples).extend_from_slice(&EMBEDDED_DATA);
        }
    }
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn entries_identical(left: &KnowledgeEntry, right: &KnowledgeEntry) -> bool {
    if left.id != right.id || left.text != right.text {
        return false;
    }
    match (&left.metadata, &right.metadata) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            let mut pending = vec![(left, right)];
            while let Some((left, right)) = pending.pop() {
                use serde_json::Value;
                match (left, right) {
                    (Value::Null, Value::Null) => {}
                    (Value::Bool(a), Value::Bool(b)) if a == b => {}
                    (Value::String(a), Value::String(b)) if a == b => {}
                    (Value::Number(a), Value::Number(b)) if a.to_string() == b.to_string() => {}
                    (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
                        pending.extend(a.iter().zip(b))
                    }
                    (Value::Object(a), Value::Object(b)) if a.len() == b.len() => {
                        for ((ka, va), (kb, vb)) in a.iter().zip(b) {
                            if ka != kb {
                                return false;
                            }
                            pending.push((va, vb));
                        }
                    }
                    _ => return false,
                }
            }
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn snapshots_keep_values_across_every_mutator() {
        let mut original = KnowledgeBase::new();
        original.add_example("first".into(), "answer", 1.0);
        let snapshot = original.clone();
        assert!(std::ptr::eq(
            original.get_examples(),
            snapshot.get_examples()
        ));
        original.push_example(TrainingExample {
            input: "second".into(),
            output: "two".into(),
            weight: 2.0,
            metadata: Some(serde_json::json!({"scope": "local"})),
        });
        original.remove_example(0).unwrap();
        original.merge(&snapshot);
        assert_eq!(snapshot.get_examples().len(), 1);
        assert_eq!(snapshot.get_examples()[0].input, "first");
        assert_eq!(original.get_examples().len(), 2);
        assert_eq!(original.get_examples()[0].input, "second");
        assert_eq!(original.get_examples()[1].input, "first");
        let mut other = snapshot.clone();
        other.add_example("third".into(), "three", 3.0);
        assert_eq!(snapshot.get_examples().len(), 1);
    }

    #[test]
    fn failed_or_empty_changes_preserve_shared_snapshots() {
        let mut original = KnowledgeBase::new();
        original.add_example("one".into(), "answer", 1.0);
        let snapshot = original.clone();
        assert!(original.remove_example(1).is_err());
        original.merge(&KnowledgeBase::new());
        assert!(std::ptr::eq(
            original.get_examples(),
            snapshot.get_examples()
        ));
        let embedded = KnowledgeBase::from_embedded();
        assert!(std::ptr::eq(
            embedded.get_examples(),
            EMBEDDED_DATA.as_slice()
        ));
        let mut merged = snapshot.clone();
        merged.merge_embedded();
        assert_eq!(merged.get_examples().len(), 1 + EMBEDDED_DATA.len());
        assert_eq!(snapshot.get_examples().len(), 1);
    }

    #[test]
    fn bounded_load_retains_format_errors_and_original_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("training.json");
        let mut original = KnowledgeBase::new();
        original.add_example("Grüße 東京".into(), "réponse", 1.5);
        let bytes = serde_json::to_vec_pretty(original.get_examples()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(KnowledgeBase::load_bounded(path.clone(), bytes.len() - 1).is_err());
        let mut loaded = KnowledgeBase::load_bounded(path.clone(), bytes.len()).unwrap();
        assert_eq!(loaded.get_examples()[0].input, "Grüße 東京");
        let snapshot = loaded.clone();
        loaded.add_example("new".into(), "answer", 1.0);
        assert_eq!(snapshot.get_examples().len(), 1);
        // Directory fsync is checked natively; Miri cannot emulate macOS
        // F_FULLFSYNC on a directory descriptor. Input/snapshot checks run there.
        #[cfg(not(miri))]
        snapshot.save(None).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::write(&path, br#"[{"input":"a","output":"b","weight":-1}]"#).unwrap();
        assert!(KnowledgeBase::load_bounded(path.clone(), 1024).is_err());
        assert!(KnowledgeBase::load(path.clone()).is_err());
        assert_eq!(
            std::fs::read(&path).unwrap(),
            br#"[{"input":"a","output":"b","weight":-1}]"#
        );
        assert!(KnowledgeBase::load_bounded(dir.path().to_path_buf(), 1024).is_err());
        std::fs::write(&path, [0xFF]).unwrap();
        assert!(matches!(KnowledgeBase::load_bounded(path.clone(), 1),
            Err(AgentError::IoError(error)) if error.kind() == std::io::ErrorKind::InvalidData));
        assert_eq!(std::fs::read(path).unwrap(), [0xFF]);
    }

    #[test]
    fn test_new_creates_empty() {
        let kb = KnowledgeBase::new();
        assert!(kb.get_examples().is_empty());
    }

    #[test]
    fn test_default_creates_empty() {
        let kb = KnowledgeBase::default();
        assert!(kb.get_examples().is_empty());
    }

    #[test]
    fn test_from_embedded() {
        let kb = KnowledgeBase::from_embedded();

        let _ = kb.get_examples();
    }

    #[test]
    fn test_add_example() {
        let mut kb = KnowledgeBase::new();
        kb.add_example("hello".to_string(), "world", 1.0);
        assert_eq!(kb.get_examples().len(), 1);
        assert_eq!(kb.get_examples()[0].input, "hello");
        assert_eq!(String::from(kb.get_examples()[0].output.clone()), "world");
    }

    #[test]
    fn test_add_multiple_examples() {
        let mut kb = KnowledgeBase::new();
        kb.add_example("q1".to_string(), "a1", 1.0);
        kb.add_example("q2".to_string(), "a2", 2.0);
        assert_eq!(kb.get_examples().len(), 2);
        assert_eq!(kb.get_examples()[1].weight, 2.0);
    }

    #[test]
    fn test_remove_valid_index() {
        let mut kb = KnowledgeBase::new();
        kb.add_example("q1".to_string(), "a1", 1.0);
        kb.add_example("q2".to_string(), "a2", 1.0);
        let removed = kb.remove_example(0).unwrap();
        assert_eq!(removed.input, "q1");
        assert_eq!(kb.get_examples().len(), 1);
        assert_eq!(kb.get_examples()[0].input, "q2");
    }

    #[test]
    fn test_remove_invalid_index() {
        let mut kb = KnowledgeBase::new();
        kb.add_example("q1".to_string(), "a1", 1.0);
        let result = kb.remove_example(5);
        assert!(result.is_err());
        match result.unwrap_err() {
            AgentError::IndexOutOfBounds(idx) => assert_eq!(idx, 5),
            other => panic!("Expected IndexOutOfBounds, got: {}", other),
        }
    }

    #[test]
    fn test_get_examples_order() {
        let mut kb = KnowledgeBase::new();
        kb.add_example("first".to_string(), "1", 1.0);
        kb.add_example("second".to_string(), "2", 1.0);
        kb.add_example("third".to_string(), "3", 1.0);
        assert_eq!(kb.get_examples()[0].input, "first");
        assert_eq!(kb.get_examples()[1].input, "second");
        assert_eq!(kb.get_examples()[2].input, "third");
    }

    #[test]
    fn test_merge_into_empty() {
        let mut kb1 = KnowledgeBase::new();
        let mut kb2 = KnowledgeBase::new();
        kb2.add_example("q1".to_string(), "a1", 1.0);

        kb1.merge(&kb2);
        assert_eq!(kb1.get_examples().len(), 1);
        assert_eq!(kb1.get_examples()[0].input, "q1");
    }

    #[test]
    fn test_merge_both_populated() {
        let mut kb1 = KnowledgeBase::new();
        kb1.add_example("q1".to_string(), "a1", 1.0);

        let mut kb2 = KnowledgeBase::new();
        kb2.add_example("q2".to_string(), "a2", 1.0);

        kb1.merge(&kb2);
        assert_eq!(kb1.get_examples().len(), 2);
        assert_eq!(kb1.get_examples()[0].input, "q1");
        assert_eq!(kb1.get_examples()[1].input, "q2");
    }

    #[test]
    fn test_merge_empty_into_populated() {
        let mut kb1 = KnowledgeBase::new();
        kb1.add_example("q1".to_string(), "a1", 1.0);
        let kb2 = KnowledgeBase::new();

        kb1.merge(&kb2);
        assert_eq!(kb1.get_examples().len(), 1);
    }

    #[test]
    fn test_merge_preserves_order() {
        let mut kb1 = KnowledgeBase::new();
        kb1.add_example("a".to_string(), "1", 1.0);
        kb1.add_example("b".to_string(), "2", 1.0);

        let mut kb2 = KnowledgeBase::new();
        kb2.add_example("c".to_string(), "3", 1.0);
        kb2.add_example("d".to_string(), "4", 1.0);

        kb1.merge(&kb2);
        let inputs: Vec<&str> = kb1
            .get_examples()
            .iter()
            .map(|e| e.input.as_str())
            .collect();
        assert_eq!(inputs, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut kb = KnowledgeBase::new();
        kb.add_example("hello".to_string(), "world", 1.5);
        kb.add_example("foo".to_string(), "bar", 2.0);

        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        kb.save(Some(path.clone())).unwrap();

        let loaded = KnowledgeBase::load(path).unwrap();
        assert_eq!(loaded.get_examples().len(), 2);
        assert_eq!(loaded.get_examples()[0].input, "hello");
        assert_eq!(loaded.get_examples()[1].weight, 2.0);
    }

    #[test]
    fn test_load_legacy_format() {
        let legacy_json = r#"[
            {"input": "hello", "output": "world", "weight": 1.0},
            {"input": "foo", "output": "bar", "weight": 2.0}
        ]"#;

        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(legacy_json.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let kb = KnowledgeBase::load(tmp.path().to_path_buf()).unwrap();
        assert_eq!(kb.get_examples().len(), 2);
        assert_eq!(kb.get_examples()[0].input, "hello");
        assert_eq!(String::from(kb.get_examples()[0].output.clone()), "world");
    }

    #[test]
    fn test_load_invalid_json() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"not valid json!!!").unwrap();
        tmp.flush().unwrap();

        let result = KnowledgeBase::load(tmp.path().to_path_buf());
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, AgentError::SerializationError(_)));
    }

    #[test]
    fn test_save_no_path() {
        let kb = KnowledgeBase::new();
        let result = kb.save(None);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, AgentError::InvalidInputError(_)));
    }
}
