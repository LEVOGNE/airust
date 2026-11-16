use crate::knowledge::KnowledgeEntry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

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

    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let entry_ids = match (self.entry_ids.is_empty(), other.entry_ids.is_empty()) {
            (true, _) => other.entry_ids.clone(),
            (_, true) => self.entry_ids.clone(),
            (false, false) => {
                let ids: BTreeSet<_> = self
                    .entry_ids
                    .intersection(&other.entry_ids)
                    .cloned()
                    .collect();
                if ids.is_empty() {
                    return None;
                }
                ids
            }
        };
        let mut metadata = self.metadata.clone();
        for (key, value) in &other.metadata {
            if metadata.get(key).is_some_and(|existing| existing != value) {
                return None;
            }
            metadata.insert(key.clone(), value.clone());
        }
        Some(Self {
            entry_ids,
            metadata,
        })
    }
}
