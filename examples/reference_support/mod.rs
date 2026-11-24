use airust::{
    documents::{DocumentLimits, DocumentSnapshot, Passage, SourceFingerprint, TextSpace},
    execution::WorkControl,
    mcp::{McpLimits, McpServer},
    retrieval::{ExcerptOptions, LexicalSource, RetrievalOptions, Retriever},
    Context, Engine,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path},
};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub const RECIPE: &str = "utf8-markdown-blank-paragraph-v1";
pub const MAX_FILE: usize = 4 * 1024 * 1024;
pub const MAX_TOTAL: usize = 32 * 1024 * 1024;
pub const MAX_JSON: usize = 64 * 1024 * 1024;
pub const MAX_REPLY: usize = 64 * 1024;

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn bounded_read(path: &Path, limit: usize) -> Result<Vec<u8>> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err("expected a regular, non-symlink file".into());
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err("file exceeds the configured byte budget".into());
    }
    Ok(bytes)
}

pub fn paragraph_spans(text: &str, maximum: usize) -> Vec<(usize, usize)> {
    assert!(maximum >= 4);
    let mut result = Vec::new();
    let mut offset = 0;
    for part in text.split("\n\n") {
        let mut rest = part.trim();
        let mut start = offset + part.len() - part.trim_start().len();
        while !rest.is_empty() {
            let mut cut = maximum.min(rest.len());
            while !rest.is_char_boundary(cut) {
                cut -= 1;
            }
            if cut < rest.len() {
                if let Some(at) = rest[..cut].rfind([' ', '\n']) {
                    if at >= maximum / 2 {
                        cut = at;
                    }
                }
            }
            let fragment = rest[..cut].trim_end();
            if !fragment.is_empty() {
                result.push((start, start + fragment.len()));
            }
            let tail = &rest[cut..];
            start += cut + tail.len() - tail.trim_start().len();
            rest = tail.trim_start();
        }
        offset += part.len() + 2;
    }
    result
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredDocument {
    name: String,
    snapshot: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredWorkspace {
    schema_version: u32,
    collection: String,
    documents: Vec<StoredDocument>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    payload_sha256: String,
    payload: String,
}

pub struct Workspace {
    pub collection: String,
    pub index: Engine,
    pub snapshots: BTreeMap<String, DocumentSnapshot>,
    pub passages: BTreeMap<String, Passage>,
}

fn checked_label(label: &str) -> Result<()> {
    if label.trim().is_empty() || label.len() > 128 || label.chars().any(char::is_control) {
        return Err("collection must contain 1..128 UTF-8 bytes without control characters".into());
    }
    Ok(())
}

fn checked_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 768
        || name.chars().any(char::is_control)
        || name.contains('\\')
        || Path::new(name)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("invalid relative Markdown source name".into());
    }
    Ok(())
}

impl Workspace {
    pub fn identity(&self) -> String {
        let revisions: Vec<_> = self
            .snapshots
            .values()
            .map(|snapshot| (snapshot.document_id(), snapshot.revision()))
            .collect();
        sha256(
            &serde_json::to_vec(&(1u32, &self.collection, RECIPE, revisions))
                .expect("string identity serialization"),
        )
    }

    pub fn from_texts(collection: &str, texts: Vec<(String, String)>) -> Result<Self> {
        checked_label(collection)?;
        if texts.is_empty() || texts.len() > 256 {
            return Err("a workspace requires 1..256 Markdown files".into());
        }
        let limits = DocumentLimits::default();
        let mut documents = Vec::new();
        let mut total = 0usize;
        for (name, text) in texts {
            checked_name(&name)?;
            total = total
                .checked_add(text.len())
                .ok_or("source byte overflow")?;
            if text.len() > MAX_FILE || total > MAX_TOTAL {
                return Err("Markdown source byte budget exceeded".into());
            }
            let snapshot = DocumentSnapshot::new(
                &format!("{collection}/{name}"),
                SourceFingerprint::from_bytes(text.as_bytes(), &limits)?,
                RECIPE,
                TextSpace::Supplied,
                text,
                &limits,
            )?;
            documents.push(StoredDocument {
                name,
                snapshot: snapshot.to_json(MAX_JSON)?,
            });
        }
        Self::restore(StoredWorkspace {
            schema_version: 1,
            collection: collection.to_owned(),
            documents,
        })
    }

    fn restore(stored: StoredWorkspace) -> Result<Self> {
        checked_label(&stored.collection)?;
        if stored.schema_version != 1 || stored.documents.is_empty() || stored.documents.len() > 256
        {
            return Err("unsupported workspace version or document count".into());
        }
        let limits = DocumentLimits::default();
        let mut snapshots = BTreeMap::new();
        let mut passages = BTreeMap::new();
        let mut entries = Vec::new();
        let mut total = 0usize;
        for document in stored.documents {
            checked_name(&document.name)?;
            let snapshot = DocumentSnapshot::from_json(&document.snapshot, &limits)?;
            if snapshot.document_id() != format!("{}/{}", stored.collection, document.name)
                || snapshot.recipe() != RECIPE
                || snapshot.text_space() != TextSpace::Supplied
                || !snapshot.pages().is_empty()
            {
                return Err("document binding does not match workspace identity".into());
            }
            snapshot.verify_source(snapshot.text().as_bytes())?;
            total = total
                .checked_add(snapshot.text().len())
                .ok_or("source byte overflow")?;
            if snapshot.text().len() > MAX_FILE || total > MAX_TOTAL {
                return Err("workspace source byte budget exceeded".into());
            }
            for (start, end) in paragraph_spans(snapshot.text(), 1800) {
                let passage = snapshot.passage(start, end)?;
                let mut entry = passage.to_entry(&snapshot)?;
                entry
                    .metadata
                    .as_mut()
                    .and_then(Value::as_object_mut)
                    .ok_or("missing document descriptor")?
                    .insert("collection".into(), json!(stored.collection));
                passage.validate_entry(&snapshot, &entry)?;
                passages.insert(entry.id.clone(), passage);
                entries.push(entry);
            }
            if snapshots
                .insert(snapshot.document_id().to_owned(), snapshot)
                .is_some()
            {
                return Err("duplicate source document".into());
            }
        }
        if entries.is_empty() {
            return Err("workspace contains no nonblank passages".into());
        }
        Ok(Self {
            collection: stored.collection,
            index: Engine::from_entries(entries)?,
            snapshots,
            passages,
        })
    }

    pub fn import(directory: &Path, collection: &str) -> Result<Self> {
        fn walk(
            root: &Path,
            path: &Path,
            files: &mut Vec<(String, String)>,
            total: &mut usize,
        ) -> Result<()> {
            if path.strip_prefix(root)?.components().count() > 32 {
                return Err("Markdown directory depth budget exceeded".into());
            }
            if !fs::symlink_metadata(path)?.is_dir() {
                return Err("Markdown input must be a non-symlink directory".into());
            }
            let mut children: Vec<_> = fs::read_dir(path)?.collect::<std::io::Result<_>>()?;
            children.sort_by_key(|entry| entry.file_name());
            for child in children {
                if child
                    .file_name()
                    .to_str()
                    .ok_or("source name is not UTF-8")?
                    .starts_with('.')
                {
                    continue;
                }
                let kind = child.file_type()?;
                if kind.is_symlink() {
                    return Err("symlink sources are not imported".into());
                }
                if kind.is_dir() {
                    walk(root, &child.path(), files, total)?;
                } else if kind.is_file() && child.path().extension().is_some_and(|ext| ext == "md")
                {
                    if files.len() >= 256 {
                        return Err("Markdown file count budget exceeded".into());
                    }
                    let bytes = bounded_read(&child.path(), MAX_FILE)?;
                    *total = total
                        .checked_add(bytes.len())
                        .ok_or("source byte overflow")?;
                    if *total > MAX_TOTAL {
                        return Err("total source byte budget exceeded".into());
                    }
                    let name = child
                        .path()
                        .strip_prefix(root)?
                        .components()
                        .map(|part| part.as_os_str().to_str().ok_or("source name is not UTF-8"))
                        .collect::<std::result::Result<Vec<_>, _>>()?
                        .join("/");
                    files.push((name, String::from_utf8(bytes)?));
                }
            }
            Ok(())
        }
        let mut texts = Vec::new();
        walk(directory, directory, &mut texts, &mut 0)?;
        Self::from_texts(collection, texts)
    }

    pub fn save(&self, directory: &Path) -> Result<()> {
        let documents = self
            .snapshots
            .values()
            .map(|snapshot| {
                let name = snapshot
                    .document_id()
                    .strip_prefix(&format!("{}/", self.collection))
                    .ok_or("foreign document")?
                    .to_owned();
                Ok(StoredDocument {
                    name,
                    snapshot: snapshot.to_json(MAX_JSON)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let payload = serde_json::to_string(&StoredWorkspace {
            schema_version: 1,
            collection: self.collection.clone(),
            documents,
        })?;
        let bytes = serde_json::to_vec(&Envelope {
            payload_sha256: sha256(payload.as_bytes()),
            payload,
        })?;
        if bytes.len() > MAX_JSON {
            return Err("encoded workspace exceeds its byte budget".into());
        }
        fs::create_dir(directory)?;
        let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
        temporary.write_all(&bytes)?;
        temporary.as_file().sync_all()?;
        temporary.persist_noclobber(directory.join("workspace.json"))?;
        Ok(())
    }

    pub fn load(directory: &Path) -> Result<Self> {
        if !fs::symlink_metadata(directory)?.is_dir() {
            return Err("workspace must be a non-symlink directory".into());
        }
        let bytes = bounded_read(&directory.join("workspace.json"), MAX_JSON)?;
        let envelope: Envelope = serde_json::from_slice(&bytes)?;
        if sha256(envelope.payload.as_bytes()) != envelope.payload_sha256 {
            return Err("workspace checksum mismatch".into());
        }
        Self::restore(serde_json::from_str(&envelope.payload)?)
    }

    pub fn locate(&self, hit: &airust::retrieval::RetrievalHit) -> Result<Value> {
        let parent = self.passages.get(&hit.id).ok_or("unknown passage ID")?;
        let snapshot = self
            .snapshots
            .get(parent.document_id())
            .ok_or("missing document revision")?;
        parent.validate_entry(
            snapshot,
            self.index.get(&hit.id).ok_or("missing source entry")?,
        )?;
        let located = parent.subspan(snapshot, hit.span.start, hit.span.end)?;
        if located.resolve(snapshot)? != hit.excerpt {
            return Err("returned excerpt does not resolve to the original document bytes".into());
        }
        Ok(
            json!({"entry_id": hit.id, "document_id": located.document_id(), "revision": located.revision(), "start_byte": located.span().start_byte(), "end_byte": located.span().end_byte(), "page_number": located.page_number()}),
        )
    }

    pub fn query(&self, query: &str, control: WorkControl) -> Result<Value> {
        self.query_using(query, control, |query, options| {
            Ok(Retriever::new(LexicalSource::new(&self.index)).query(query, options)?)
        })
    }

    pub fn query_using(
        &self,
        query: &str,
        control: WorkControl,
        mut retrieve: impl FnMut(
            &str,
            &RetrievalOptions,
        ) -> Result<airust::retrieval::RetrievalResponse>,
    ) -> Result<Value> {
        let mut found_evidence = false;
        for limit in (0..=6).rev() {
            let options = RetrievalOptions::new(limit)
                .with_candidates(100)?
                .with_byte_budgets(4096, 12000, 2000, 48 * 1024)?
                .with_context(Context::new().with_metadata("collection", self.collection.as_str()))
                .with_relevant_excerpts(ExcerptOptions::default().with_paragraph_boundaries())
                .with_distinct_evidence()
                .with_control(control.clone());
            control
                .checkpoint()
                .map_err(airust::retrieval::RetrievalError::from)?;
            let response = retrieve(query, &options)?;
            control
                .checkpoint()
                .map_err(airust::retrieval::RetrievalError::from)?;
            found_evidence |= !response.hits().is_empty();
            let citations = response
                .hits()
                .iter()
                .map(|hit| self.locate(hit))
                .collect::<Result<Vec<_>>>()?;
            let result = json!({"schema_version": 1, "collection": self.collection, "answerability": if found_evidence {"not_assessed"} else {"no_evidence"}, "source_integrity": "verified", "truncated": limit < 6 || response.truncated(), "retrieval": response, "citations": citations});
            if serde_json::to_vec(&result)?.len() <= MAX_REPLY {
                return Ok(result);
            }
        }
        Err("complete workspace response exceeds its JSON budget".into())
    }

    pub fn into_server(self) -> Result<McpServer> {
        Ok(McpServer::new(
            self.index.entries().cloned(),
            self.snapshots.into_values(),
            McpLimits {
                max_hits: 6,
                max_excerpt_bytes: 2000,
                max_result_bytes: MAX_REPLY,
                ..Default::default()
            },
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use airust::{
        execution::{CancellationToken, ExecutionError},
        retrieval::RetrievalError,
    };

    fn sample() -> Workspace {
        Workspace::from_texts(
            "manuals",
            vec![(
                "Grüße.md".into(),
                "Einleitung.\n\nCafe\u{301} — Rust schützt Speicher.\n\n東京 bleibt erhalten."
                    .into(),
            )],
        )
        .unwrap()
    }

    #[test]
    fn independent_python_corpus_and_rust_snapshot_passages_agree() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reference-v1");
        let workspace = Workspace::import(&fixture.join("sources"), "airust-reference").unwrap();
        let mut rows: Vec<_> = workspace.passages.values().map(|passage| {
            let document = passage.document_id().strip_prefix("airust-reference/").unwrap();
            let snapshot = &workspace.snapshots[passage.document_id()];
            let start = passage.span().start_byte();
            let end = passage.span().end_byte();
            json!({"id": format!("ref-v1:{document}:{start}:{end}"), "document": document, "start": start, "end": end, "text": passage.resolve(snapshot).unwrap(), "collection": "airust-reference"})
        }).collect();
        rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        let guards: Vec<_> = rows
            .iter()
            .take(16)
            .map(|row| {
                let mut guard = row.clone();
                guard["id"] =
                    json!(row["id"]
                        .as_str()
                        .unwrap()
                        .replacen("ref-v1:", "guard-v1:", 1));
                guard["collection"] = json!("out-of-scope-fixture");
                guard
            })
            .collect();
        rows.extend(guards);
        let mut encoded = String::new();
        for row in &rows {
            encoded.push_str(&serde_json::to_string(row).unwrap());
            encoded.push('\n');
        }
        let manifest: Value =
            serde_json::from_slice(&fs::read(fixture.join("fixture-manifest.json")).unwrap())
                .unwrap();
        assert_eq!(
            rows.len(),
            manifest["corpus_entries"].as_u64().unwrap() as usize
        );
        assert_eq!(
            sha256(encoded.as_bytes()),
            manifest["artifacts"]["corpus.jsonl"]["sha256"]
        );
    }

    #[test]
    fn workspace_round_trip_preserves_original_unicode_citations_and_bounds() {
        let temporary = tempfile::tempdir().unwrap();
        let destination = temporary.path().join("workspace");
        let original = sample();
        original.save(&destination).unwrap();
        let restored = Workspace::load(&destination).unwrap();
        assert_eq!(
            original.index.to_json().unwrap(),
            restored.index.to_json().unwrap()
        );
        let result = restored
            .query("Café Rust", WorkControl::new(CancellationToken::new()))
            .unwrap();
        assert_eq!(result["source_integrity"], "verified");
        assert_eq!(result["answerability"], "not_assessed");
        let hits = result["retrieval"]["hits"].as_array().unwrap();
        assert!(!hits.is_empty() && hits.len() <= 6);
        assert!(hits[0]["excerpt"].as_str().unwrap().contains("Cafe\u{301}"));
        assert!(serde_json::to_vec(&result).unwrap().len() <= MAX_REPLY);
        let citations = result["citations"].as_array().unwrap();
        for (hit, citation) in hits.iter().zip(citations) {
            let snapshot = &restored.snapshots[citation["document_id"].as_str().unwrap()];
            let start = citation["start_byte"].as_u64().unwrap() as usize;
            let end = citation["end_byte"].as_u64().unwrap() as usize;
            assert_eq!(
                &snapshot.text()[start..end],
                hit["excerpt"].as_str().unwrap()
            );
        }
        assert_eq!(restored.into_server().unwrap().stats().documents, 1);
    }

    #[test]
    fn save_never_replaces_an_existing_workspace() {
        let temporary = tempfile::tempdir().unwrap();
        let destination = temporary.path().join("workspace");
        let workspace = sample();
        workspace.save(&destination).unwrap();
        let before = fs::read(destination.join("workspace.json")).unwrap();
        assert!(workspace.save(&destination).is_err());
        assert_eq!(
            fs::read(destination.join("workspace.json")).unwrap(),
            before
        );
    }

    #[test]
    fn changed_workspace_payload_is_rejected() {
        let temporary = tempfile::tempdir().unwrap();
        let destination = temporary.path().join("workspace");
        sample().save(&destination).unwrap();
        let file = destination.join("workspace.json");
        let mut value: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        value["payload"] = json!(value["payload"]
            .as_str()
            .unwrap()
            .replace("manuals", "changed"));
        fs::write(file, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(Workspace::load(&destination).is_err());
    }

    #[test]
    fn cancellation_returns_a_typed_error_without_partial_success() {
        let token = CancellationToken::new();
        token.cancel();
        let error = sample().query("Rust", WorkControl::new(token)).unwrap_err();
        assert!(matches!(
            error.downcast_ref::<RetrievalError>(),
            Some(RetrievalError::Execution(ExecutionError::Cancelled))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_sources_are_rejected() {
        let temporary = tempfile::tempdir().unwrap();
        let input = temporary.path().join("input");
        fs::create_dir(&input).unwrap();
        let outside = temporary.path().join("outside.md");
        fs::write(&outside, "private outside content").unwrap();
        std::os::unix::fs::symlink(&outside, input.join("linked.md")).unwrap();
        assert!(Workspace::import(&input, "manuals").is_err());
    }
}
