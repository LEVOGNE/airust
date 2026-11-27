#![doc = include_str!("../../docs/LOCAL-RERANKING.md")]

use super::{RerankError, Reranker};
use crate::model_assets::{self, Asset};
use fastembed::{
    AutoDevicePolicy, OnnxSource, RerankInitOptionsUserDefined, TextRerank, TokenizerFiles,
    UserDefinedRerankingModel,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

#[derive(Deserialize)]
struct Manifest {
    repository: String,
    files: BTreeMap<String, Asset>,
}

pub struct FastEmbedReranker {
    recipe: String,
    model: TextRerank,
    // Fields drop in declaration order: keep the private file until native state
    // has released any file handles/mappings, including on Windows.
    _snapshot: Option<tempfile::NamedTempFile>,
}

impl FastEmbedReranker {
    pub fn multilingual_minilm_v2(
        model_directory: impl AsRef<Path>,
        runtime_library: impl AsRef<Path>,
        threads: usize,
    ) -> Result<Self, RerankError> {
        Self::load(
            model_directory.as_ref(),
            runtime_library.as_ref(),
            threads,
            None,
        )
    }

    /// Streams the pinned model into a private verified temporary file for ONNX
    /// Runtime. The original assets remain untouched. Requires enough free disk
    /// space for the model; the snapshot is removed when this scorer is dropped.
    /// Abrupt process termination can leave the temporary file behind.
    pub fn multilingual_minilm_v2_from_snapshot(
        model_directory: impl AsRef<Path>,
        runtime_library: impl AsRef<Path>,
        threads: usize,
        snapshot_directory: impl AsRef<Path>,
    ) -> Result<Self, RerankError> {
        Self::load(
            model_directory.as_ref(),
            runtime_library.as_ref(),
            threads,
            Some(snapshot_directory.as_ref()),
        )
    }

    fn load(
        model_directory: &Path,
        runtime_library: &Path,
        threads: usize,
        snapshot_directory: Option<&Path>,
    ) -> Result<Self, RerankError> {
        if threads == 0 {
            return Err(RerankError::Configuration("thread count must be positive"));
        }
        let manifest: Manifest =
            serde_json::from_str(include_str!("../../docs/models/mmarco-minilm-v2.json"))
                .map_err(provider_error)?;
        let mut fingerprint = Sha256::new();
        fingerprint.update(b"airust/rerank/raw-single-logit/pair-longest-first/max512/v1");
        let expected_model = manifest
            .files
            .get("onnx/model.onnx")
            .ok_or(RerankError::Output("missing packaged asset manifest"))?;
        let (model, snapshot) = read_model(
            &model_directory.join("onnx/model.onnx"),
            expected_model,
            &mut fingerprint,
            snapshot_directory,
        )?;
        let mut read = |name: &str| -> Result<Vec<u8>, RerankError> {
            let expected = manifest
                .files
                .get(name)
                .ok_or(RerankError::Output("missing packaged asset manifest"))?;
            let bytes = model_assets::read(&model_directory.join(name), expected)
                .map_err(provider_error)?;
            fingerprint.update((name.len() as u64).to_le_bytes());
            fingerprint.update(name.as_bytes());
            fingerprint.update((bytes.len() as u64).to_le_bytes());
            fingerprint.update(&bytes);
            Ok(bytes)
        };
        let tokenizer = TokenizerFiles {
            tokenizer_file: read("tokenizer.json")?,
            config_file: read("config.json")?,
            special_tokens_map_file: read("special_tokens_map.json")?,
            tokenizer_config_file: read("tokenizer_config.json")?,
        };
        let recipe = format!(
            "{}:sha256:{:x}",
            manifest.repository,
            fingerprint.finalize()
        );
        let environment =
            crate::local_runtime::environment(runtime_library).map_err(provider_error)?;
        let model = TextRerank::try_new_from_user_defined(
            UserDefinedRerankingModel::new(model, tokenizer),
            RerankInitOptionsUserDefined::new()
                .with_environment(environment)
                .with_max_length(512)
                .with_intra_threads(threads)
                .with_auto_device(AutoDevicePolicy::PreferCPU),
        )
        .map_err(provider_error)?;
        Ok(Self {
            recipe,
            model,
            _snapshot: snapshot,
        })
    }
}

impl Reranker for FastEmbedReranker {
    fn recipe(&self) -> &str {
        &self.recipe
    }

    fn score(&mut self, query: &str, documents: &[&str]) -> Result<Vec<f64>, RerankError> {
        if query.trim().is_empty() {
            return Err(RerankError::EmptyQuery);
        }
        if documents.iter().any(|text| text.trim().is_empty()) {
            return Err(RerankError::Output("empty document passed to local scorer"));
        }
        if documents.is_empty() {
            return Ok(Vec::new());
        }
        let ranked = self
            .model
            .rerank(query, documents, false, Some(16))
            .map_err(inference_error)?;
        if ranked.len() != documents.len() {
            return Err(RerankError::ScoreCount {
                expected: documents.len(),
                actual: ranked.len(),
            });
        }
        let mut scores = vec![None; documents.len()];
        for result in ranked {
            let value = scores
                .get_mut(result.index)
                .filter(|slot| slot.is_none())
                .ok_or(RerankError::Output(
                    "duplicate or out-of-range document index",
                ))?;
            if !result.score.is_finite() {
                return Err(RerankError::NonFiniteScore(result.index + 1));
            }
            *value = Some(f64::from(result.score));
        }
        scores
            .into_iter()
            .map(|value| value.ok_or(RerankError::Output("missing document score")))
            .collect()
    }
}

fn read_model(
    path: &Path,
    expected: &Asset,
    fingerprint: &mut Sha256,
    snapshot_directory: Option<&Path>,
) -> Result<(OnnxSource, Option<tempfile::NamedTempFile>), RerankError> {
    const NAME: &str = "onnx/model.onnx";
    fingerprint.update((NAME.len() as u64).to_le_bytes());
    fingerprint.update(NAME.as_bytes());
    fingerprint.update(expected.bytes.to_le_bytes());
    match snapshot_directory {
        None => {
            let bytes = model_assets::read(path, expected).map_err(provider_error)?;
            fingerprint.update(&bytes);
            Ok((OnnxSource::Memory(bytes), None))
        }
        Some(directory) => {
            let snapshot = model_assets::snapshot(path, expected, directory, ".onnx", |chunk| {
                fingerprint.update(chunk);
            })
            .map_err(provider_error)?;
            // Load the verified copy, not a re-opened original that might have
            // changed after verification. The directory is application-owned.
            let source = OnnxSource::File(snapshot.path().to_path_buf());
            Ok((source, Some(snapshot)))
        }
    }
}

fn provider_error(error: impl std::fmt::Display) -> RerankError {
    RerankError::Provider(error.to_string())
}

fn inference_error(error: fastembed::Error) -> RerankError {
    match error {
        fastembed::Error::InvalidShape(_)
        | fastembed::Error::OutputKeyMissing { .. }
        | fastembed::Error::TensorExtraction(_) => {
            RerankError::Output("malformed native model output")
        }
        other => provider_error(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_native_outputs_remain_protocol_errors_instead_of_enabling_fallback() {
        for error in [
            fastembed::Error::InvalidShape("wrong count".into()),
            fastembed::Error::OutputKeyMissing {
                key: "logits".into(),
            },
            fastembed::Error::TensorExtraction("wrong type".into()),
        ] {
            assert!(matches!(inference_error(error), RerankError::Output(_)));
        }
        assert!(matches!(
            inference_error(fastembed::Error::OrtSession("unavailable".into())),
            RerankError::Provider(_)
        ));
    }

    #[test]
    fn model_source_modes_preserve_recipe_framing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model");
        std::fs::write(&path, b"abc").unwrap();
        let expected = Asset {
            bytes: 3,
            sha256: format!("{:x}", Sha256::digest(b"abc")),
        };
        let mut memory_hash = Sha256::new();
        let (memory, _) = read_model(&path, &expected, &mut memory_hash, None).unwrap();
        assert!(matches!(memory, OnnxSource::Memory(bytes) if bytes == b"abc"));
        let mut file_hash = Sha256::new();
        let (source, snapshot) =
            read_model(&path, &expected, &mut file_hash, Some(dir.path())).unwrap();
        assert_eq!(memory_hash.finalize(), file_hash.finalize());
        let OnnxSource::File(copy) = source else {
            panic!("expected snapshot");
        };
        assert_eq!(std::fs::read(&copy).unwrap(), b"abc");
        drop(snapshot);
        assert!(!copy.exists());
    }

    #[test]
    fn invalid_threads_and_missing_files_return_errors_without_downloads() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            FastEmbedReranker::multilingual_minilm_v2(dir.path(), "missing-runtime", 0),
            Err(RerankError::Configuration(_))
        ));
        assert!(matches!(
            FastEmbedReranker::multilingual_minilm_v2(dir.path(), "missing-runtime", 2),
            Err(RerankError::Provider(_))
        ));
    }
}
