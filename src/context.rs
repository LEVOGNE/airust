//! Explicit search scope, independent of UI state or conversation formatting.

use crate::knowledge::KnowledgeEntry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// IDs are ORed; metadata conditions are ANDed. Empty filters accept every entry.
/// Metadata filters compare exact top-level JSON values and never alter the query.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Context {
    #[serde(default)]
    pub entry_ids: BTreeSet<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_entry(mut self, id: impl Into<String>) -> Self {
        self.entry_ids.insert(id.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn includes(&self, entry: &KnowledgeEntry) -> bool {
        (self.entry_ids.is_empty() || self.entry_ids.contains(&entry.id))
            && self.metadata.iter().all(|(key, value)| {
                entry
                    .metadata
                    .as_ref()
                    .and_then(Value::as_object)
                    .and_then(|object| object.get(key))
                    == Some(value)
            })
    }
}
