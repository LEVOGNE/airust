//! Explicit local case collection. Caller declarations never become verified
//! traffic or independent judgments merely because their source spans validate.
use crate::reference_support::{
    bounded_read, encode_bounded, sha256, Result, Workspace, MAX_JSON, RECIPE,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const MAX_CASE: usize = 128 * 1024;
const MAX_CASES: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    UserSupplied,
    UserQuestionReplay,
    AssistantAuthored,
    SyntheticTest,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureRequest {
    pub case_id: String,
    pub query: String,
    pub language: String,
    pub origin: Origin,
    pub origin_evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceIdentity {
    pub collection: String,
    pub revision_sha256: String,
    pub documents: usize,
    pub passages: usize,
}
impl WorkspaceIdentity {
    pub fn of(workspace: &Workspace) -> Self {
        Self {
            collection: workspace.collection.clone(),
            revision_sha256: workspace.identity(),
            documents: workspace.snapshots.len(),
            passages: workspace.index.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRange {
    pub document_id: String,
    pub revision: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub quote: String,
}
impl SourceRange {
    fn validate(&self, workspace: &Workspace) -> Result<()> {
        let snapshot = workspace
            .snapshots
            .get(&self.document_id)
            .ok_or("unknown source document")?;
        if snapshot.revision() != self.revision
            || self.quote.is_empty()
            || self.quote.len() > 2000
            || snapshot.text().get(self.start_byte..self.end_byte) != Some(self.quote.as_str())
        {
            return Err("source range, quote or document revision differs".into());
        }
        snapshot.passage(self.start_byte, self.end_byte)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerKind {
    Human,
    Assistant,
    Test,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Independence {
    Independent,
    SameAuthor,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Answerability {
    Supported,
    Unsupported,
    Uncertain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub reviewer: String,
    pub reviewer_kind: ReviewerKind,
    pub independence: Independence,
    pub answerability: Answerability,
    pub relevant_sources: Vec<SourceRange>,
    pub notes: String,
}
impl Review {
    fn validate(&self, workspace: &Workspace) -> Result<()> {
        label(&self.reviewer, 128)?;
        if self.notes.len() > 4096
            || self.relevant_sources.len() > 32
            || self.independence == Independence::Independent
                && self.reviewer_kind != ReviewerKind::Human
            || self.answerability == Answerability::Supported && self.relevant_sources.is_empty()
            || self.answerability == Answerability::Unsupported && !self.relevant_sources.is_empty()
        {
            return Err("invalid review declaration, answerability or source count".into());
        }
        for source in &self.relevant_sources {
            source.validate(workspace)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub recipe: String,
    pub fallback_reason: Option<String>,
    pub reranking: Option<Value>,
    pub query_ms: f64,
    pub truncated: bool,
    pub evidence: Vec<SourceRange>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PilotCase {
    pub schema_version: u32,
    pub captured_unix_ms: u64,
    pub implementation_version: String,
    pub request: CaptureRequest,
    pub workspace: WorkspaceIdentity,
    pub observation: Observation,
    pub review: Option<Review>,
    pub previous_capture_sha256: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    payload_sha256: String,
    payload: String,
}

fn label(value: &str, maximum: usize) -> Result<()> {
    if value.trim().is_empty() || value.len() > maximum || value.chars().any(char::is_control) {
        return Err("label is blank, too long or contains controls".into());
    }
    Ok(())
}
fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn request_valid(request: &CaptureRequest) -> Result<()> {
    if request.case_id.is_empty()
        || request.case_id.len() > 64
        || !request
            .case_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("case ID must be 1..64 ASCII letters, digits, '-' or '_'".into());
    }
    if request.query.trim().is_empty() || request.query.len() > 4096 {
        return Err("query byte budget or blank query".into());
    }
    if request.language.len() < 2
        || request.language.len() > 32
        || !request
            .language
            .bytes()
            .all(|b| b.is_ascii_alphabetic() || b == b'-')
    {
        return Err("language must be a 2..32 byte language tag".into());
    }
    label(&request.origin_evidence, 2048)
}

pub fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if !fs::symlink_metadata(parent)?.is_dir() {
        return Err("output parent must be a non-symlink directory".into());
    }
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    file.persist_noclobber(path)?;
    Ok(())
}

pub fn read_request(path: &Path) -> Result<CaptureRequest> {
    let request = serde_json::from_slice(&bounded_read(path, 16 * 1024)?)?;
    request_valid(&request)?;
    Ok(request)
}
impl PilotCase {
    pub fn capture(
        workspace: &Workspace,
        request: CaptureRequest,
        response: &Value,
        query_ms: f64,
    ) -> Result<Self> {
        request_valid(&request)?;
        let hits = response["retrieval"]["hits"]
            .as_array()
            .ok_or("missing hits")?;
        let citations = response["citations"]
            .as_array()
            .ok_or("missing citations")?;
        if hits.len() != citations.len() {
            return Err("citation count differs".into());
        }
        let mut evidence = Vec::new();
        for (hit, citation) in hits.iter().zip(citations) {
            evidence.push(SourceRange {
                document_id: citation["document_id"]
                    .as_str()
                    .ok_or("missing document")?
                    .into(),
                revision: citation["revision"]
                    .as_str()
                    .ok_or("missing revision")?
                    .into(),
                start_byte: usize::try_from(
                    citation["start_byte"].as_u64().ok_or("missing start")?,
                )?,
                end_byte: usize::try_from(citation["end_byte"].as_u64().ok_or("missing end")?)?,
                quote: hit["excerpt"].as_str().ok_or("missing quote")?.into(),
            });
        }
        let result = Self {
            schema_version: 1,
            captured_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_millis()
                .try_into()?,
            implementation_version: airust::VERSION.into(),
            request,
            workspace: WorkspaceIdentity::of(workspace),
            observation: Observation {
                recipe: response["retrieval"]["recipe"]
                    .as_str()
                    .ok_or("missing recipe")?
                    .into(),
                fallback_reason: response["retrieval"]["fallback_reason"]
                    .as_str()
                    .map(str::to_owned),
                reranking: response["retrieval"]["reranking"]
                    .as_object()
                    .map(|_| response["retrieval"]["reranking"].clone()),
                query_ms,
                truncated: response["truncated"]
                    .as_bool()
                    .ok_or("missing truncation")?,
                evidence,
            },
            review: None,
            previous_capture_sha256: None,
        };
        result.validate(workspace, &WorkspaceIdentity::of(workspace))?;
        Ok(result)
    }
    fn validate(&self, workspace: &Workspace, identity: &WorkspaceIdentity) -> Result<()> {
        request_valid(&self.request)?;
        label(&self.implementation_version, 128)?;
        label(&self.observation.recipe, 4096)?;
        if self.schema_version != 1
            || &self.workspace != identity
            || !self.observation.query_ms.is_finite()
            || self.observation.query_ms < 0.0
            || self.observation.evidence.len() > 6
            || self
                .observation
                .evidence
                .iter()
                .map(|s| s.quote.len())
                .sum::<usize>()
                > 12000
            || self
                .previous_capture_sha256
                .as_ref()
                .is_some_and(|s| !hash(s))
            || self
                .observation
                .fallback_reason
                .as_ref()
                .is_some_and(|s| s.len() > 4096)
            || self
                .observation
                .reranking
                .as_ref()
                .is_some_and(|s| serde_json::to_vec(s).map_or(true, |b| b.len() > 8192))
        {
            return Err("case schema, workspace revision, observation or budget differs".into());
        }
        for source in &self.observation.evidence {
            source.validate(workspace)?;
        }
        if let Some(review) = &self.review {
            review.validate(workspace)?;
        }
        Ok(())
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        let payload = String::from_utf8(encode_bounded(self, MAX_CASE)?)?;
        let bytes = encode_bounded(
            &Envelope {
                payload_sha256: sha256(payload.as_bytes()),
                payload,
            },
            MAX_CASE,
        )?;
        write_new(path, &bytes)
    }
    pub fn load(workspace: &Workspace, path: &Path) -> Result<Self> {
        let envelope: Envelope = serde_json::from_slice(&bounded_read(path, MAX_CASE)?)?;
        if sha256(envelope.payload.as_bytes()) != envelope.payload_sha256 {
            return Err("case checksum mismatch".into());
        }
        let case: Self = serde_json::from_str(&envelope.payload)?;
        case.validate(workspace, &WorkspaceIdentity::of(workspace))?;
        Ok(case)
    }
    pub fn reviewed(mut self, workspace: &Workspace, review_path: &Path) -> Result<Self> {
        let review: Review = serde_json::from_slice(&bounded_read(review_path, MAX_CASE)?)?;
        review.validate(workspace)?;
        self.previous_capture_sha256 = Some(sha256(&serde_json::to_vec(&self)?));
        self.review = Some(review);
        self.validate(workspace, &WorkspaceIdentity::of(workspace))?;
        Ok(self)
    }
}

pub fn init(input: &Path, output: &Path, collection: &str) -> Result<Value> {
    let workspace = Workspace::import(input, collection)?;
    fs::create_dir(output)?;
    workspace.save(&output.join("workspace"))?;
    fs::create_dir(output.join("cases"))?;
    fs::create_dir(output.join("reviewed"))?;
    let path = fs::canonicalize(output.join("workspace"))?;
    let config = json!({"mcpServers":{"airust-pilot":{"command":std::env::current_exe()?,"args":["serve",path]}}});
    write_new(
        &output.join("mcp-client.json"),
        &serde_json::to_vec_pretty(&config)?,
    )?;
    Ok(
        json!({"schema_version":1,"pilot":fs::canonicalize(output)?,"workspace":WorkspaceIdentity::of(&workspace),"source_recipe":RECIPE,"capture":"explicit only; query and serve do not record cases","mcp_client_config":output.join("mcp-client.json")}),
    )
}

pub fn export(workspace: &Workspace, input: &Path, output: &Path) -> Result<Value> {
    if !fs::symlink_metadata(input)?.is_dir() {
        return Err("case directory must be non-symlink".into());
    }
    let mut paths = Vec::new();
    for item in fs::read_dir(input)? {
        let item = item?;
        if item.path().extension().is_some_and(|s| s == "json") {
            if !item.file_type()?.is_file() {
                return Err("case input must be a regular non-symlink file".into());
            }
            paths.push(item.path());
            if paths.len() > MAX_CASES {
                return Err("case count exceeds 256".into());
            }
        }
    }
    if paths.is_empty() {
        return Err("no pilot cases selected".into());
    }
    paths.sort();
    let mut cases = Vec::new();
    let mut ids = BTreeSet::new();
    let identity = WorkspaceIdentity::of(workspace);
    for path in paths {
        let case = PilotCase::load(workspace, &path)?;
        if !ids.insert(case.request.case_id.clone()) {
            return Err("duplicate case ID; select one immutable revision per case".into());
        }
        cases.push(case);
    }
    cases.sort_by(|a, b| a.request.case_id.cmp(&b.request.case_id));
    let mut bytes = Vec::new();
    for case in &cases {
        serde_json::to_writer(&mut bytes, case)?;
        bytes.push(b'\n');
    }
    if bytes.len() > 32 * 1024 * 1024 {
        return Err("exported cases exceed 32 MiB".into());
    }
    let user_cases = cases
        .iter()
        .filter(|c| c.request.origin == Origin::UserSupplied)
        .count();
    let replayed = cases
        .iter()
        .filter(|c| c.request.origin == Origin::UserQuestionReplay)
        .count();
    let reviewed = cases
        .iter()
        .filter(|c| {
            c.review.as_ref().is_some_and(|r| {
                r.reviewer_kind == ReviewerKind::Human
                    && r.independence == Independence::Independent
            })
        })
        .count();
    let qualified = cases
        .iter()
        .filter(|c| {
            c.request.origin == Origin::UserSupplied
                && c.review.as_ref().is_some_and(|r| {
                    r.reviewer_kind == ReviewerKind::Human
                        && r.independence == Independence::Independent
                })
        })
        .count();
    // Reserve the destination atomically. All semantic validation happens above;
    // the manifest is published last so an I/O failure is never a complete export.
    fs::create_dir(output)?;
    workspace.save(&output.join("workspace"))?;
    write_new(&output.join("cases.jsonl"), &bytes)?;
    let source = bounded_read(&output.join("workspace/workspace.json"), MAX_JSON)?;
    let manifest = json!({"format":"airust-pilot-export-v1","schema_version":1,"workspace":identity,"cases":cases.len(),"declared_user_supplied":user_cases,"user_question_replays":replayed,"declared_independent_human_reviews":reviewed,"declared_live_and_independently_reviewed":qualified,"origin_assurance":"caller declarations, not authenticated traffic or reviewer identities","evaluation_ready":false,"evaluation_readiness":if qualified>=50 {"awaiting frozen source-family split and policy"} else {"awaiting at least 50 genuine independently reviewed pilot questions"},"artifacts":{"cases.jsonl":{"sha256":sha256(&bytes),"bytes":bytes.len()},"workspace/workspace.json":{"sha256":sha256(&source),"bytes":source.len()}}});
    write_new(
        &output.join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use airust::execution::WorkControl;
    fn sample() -> Workspace {
        Workspace::from_texts(
            "pilot",
            vec![(
                "guide.md".into(),
                "Café — Rust ownership protects memory.\n\nOther notes.".into(),
            )],
        )
        .unwrap()
    }
    fn captured(workspace: &Workspace) -> PilotCase {
        let request = CaptureRequest {
            case_id: "synthetic-001".into(),
            query: "Café Rust".into(),
            language: "de".into(),
            origin: Origin::SyntheticTest,
            origin_evidence: "Explicit synthetic unit-test fixture".into(),
        };
        let response = workspace
            .query(&request.query, WorkControl::default())
            .unwrap();
        PilotCase::capture(workspace, request, &response, 1.25).unwrap()
    }
    #[test]
    fn immutable_case_round_trip_preserves_origin_and_utf8_source_ranges() {
        let workspace = sample();
        let case = captured(&workspace);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("case.json");
        case.save(&path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(case.save(&path).is_err());
        assert_eq!(bytes, fs::read(&path).unwrap());
        let restored = PilotCase::load(&workspace, &path).unwrap();
        assert_eq!(restored.request.origin, Origin::SyntheticTest);
        assert!(!restored.observation.evidence.is_empty());
        assert!(restored.review.is_none());
    }
    #[test]
    fn valid_checksum_does_not_accept_stale_revisions_or_split_utf8() {
        let workspace = sample();
        let mut case = captured(&workspace);
        let dir = tempfile::tempdir().unwrap();
        case.observation.evidence[0].start_byte = 4;
        case.save(&dir.path().join("invalid.json")).unwrap();
        assert!(PilotCase::load(&workspace, &dir.path().join("invalid.json")).is_err());
        let changed = Workspace::from_texts(
            "pilot",
            vec![("guide.md".into(), "Changed Rust source".into())],
        )
        .unwrap();
        let original = captured(&workspace);
        original.save(&dir.path().join("original.json")).unwrap();
        assert!(PilotCase::load(&changed, &dir.path().join("original.json")).is_err());
    }
    #[test]
    fn export_keeps_unreviewed_synthetic_cases_out_of_real_evaluation_counts() {
        let workspace = sample();
        let dir = tempfile::tempdir().unwrap();
        let cases = dir.path().join("cases");
        fs::create_dir(&cases).unwrap();
        captured(&workspace).save(&cases.join("one.json")).unwrap();
        let output = dir.path().join("export");
        let manifest = export(&workspace, &cases, &output).unwrap();
        assert_eq!(manifest["declared_user_supplied"], 0);
        assert_eq!(manifest["evaluation_ready"], false);
        assert!(export(&workspace, &cases, &output).is_err());
        let original = fs::read(output.join("cases.jsonl")).unwrap();
        assert_eq!(
            manifest["artifacts"]["cases.jsonl"]["sha256"],
            sha256(&original)
        );
        assert_eq!(
            Workspace::load(&output.join("workspace"))
                .unwrap()
                .identity(),
            workspace.identity()
        );
    }
    #[test]
    fn review_claims_require_exact_sources_and_human_role_for_independence() {
        let workspace = sample();
        let case = captured(&workspace);
        let source = case.observation.evidence[0].clone();
        let mut review = Review {
            reviewer: "test-reviewer".into(),
            reviewer_kind: ReviewerKind::Test,
            independence: Independence::Independent,
            answerability: Answerability::Supported,
            relevant_sources: vec![source],
            notes: "synthetic review, not human evidence".into(),
        };
        assert!(review.validate(&workspace).is_err());
        review.independence = Independence::SameAuthor;
        assert!(review.validate(&workspace).is_ok());
        review.relevant_sources[0].revision = "stale".into();
        assert!(review.validate(&workspace).is_err());
    }
    #[test]
    fn duplicate_cases_and_oversized_inputs_fail_before_creating_an_export() {
        let workspace = sample();
        let dir = tempfile::tempdir().unwrap();
        let cases = dir.path().join("cases");
        fs::create_dir(&cases).unwrap();
        let case = captured(&workspace);
        case.save(&cases.join("one.json")).unwrap();
        case.save(&cases.join("two.json")).unwrap();
        let output = dir.path().join("export");
        assert!(export(&workspace, &cases, &output).is_err());
        assert!(!output.exists());
        let mut bad = case.request;
        bad.query = "x".repeat(4097);
        assert!(request_valid(&bad).is_err());
        bad.query = "Rust".into();
        bad.case_id = "../../outside".into();
        assert!(request_valid(&bad).is_err());
    }
}
