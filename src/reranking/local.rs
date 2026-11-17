#![doc = include_str!("../../docs/LOCAL-RERANKING.md")]

use super::{RerankError, Reranker};
use fastembed::{
    AutoDevicePolicy, RerankInitOptionsUserDefined, TextRerank, TokenizerFiles,
    UserDefinedRerankingModel,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, io::Read, path::Path};

#[derive(Deserialize)]
struct Asset {
    bytes: u64,
    sha256: String,
}
#[derive(Deserialize)]
struct Manifest {
    repository: String,
    files: BTreeMap<String, Asset>,
}

pub struct FastEmbedReranker {
    recipe: String,
    model: TextRerank,
}

impl FastEmbedReranker {
    pub fn multilingual_minilm_v2(
        model_directory: impl AsRef<Path>,
        runtime_library: impl AsRef<Path>,
        threads: usize,
    ) -> Result<Self, RerankError> {
        if threads == 0 {
            return Err(RerankError::Configuration("thread count must be positive"));
        }
        let manifest: Manifest =
            serde_json::from_str(include_str!("../../docs/models/mmarco-minilm-v2.json"))
                .map_err(provider_error)?;
        let mut fingerprint = Sha256::new();
        fingerprint.update(b"airust/rerank/raw-single-logit/pair-longest-first/max512/v1");
        let mut read = |name: &str| -> Result<Vec<u8>, RerankError> {
            let expected = manifest
                .files
                .get(name)
                .ok_or(RerankError::Output("missing packaged asset manifest"))?;
            let bytes = read_asset(&model_directory.as_ref().join(name), expected)?;
            fingerprint.update((name.len() as u64).to_le_bytes());
            fingerprint.update(name.as_bytes());
            fingerprint.update((bytes.len() as u64).to_le_bytes());
            fingerprint.update(&bytes);
            Ok(bytes)
        };
        let model = read("onnx/model.onnx")?;
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
            crate::local_runtime::environment(runtime_library.as_ref()).map_err(provider_error)?;
        let model = TextRerank::try_new_from_user_defined(
            UserDefinedRerankingModel::new(model, tokenizer),
            RerankInitOptionsUserDefined::new()
                .with_environment(environment)
                .with_max_length(512)
                .with_intra_threads(threads)
                .with_auto_device(AutoDevicePolicy::PreferCPU),
        )
        .map_err(provider_error)?;
        Ok(Self { recipe, model })
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

fn read_asset(path: &Path, expected: &Asset) -> Result<Vec<u8>, RerankError> {
    let file = std::fs::File::open(path).map_err(provider_error)?;
    if file.metadata().map_err(provider_error)?.len() != expected.bytes {
        return Err(RerankError::Provider(
            "local model asset size mismatch".into(),
        ));
    }
    let limit = expected
        .bytes
        .checked_add(1)
        .ok_or(RerankError::Output("asset size overflow"))?;
    let mut bytes = Vec::new();
    file.take(limit)
        .read_to_end(&mut bytes)
        .map_err(provider_error)?;
    if bytes.len() as u64 != expected.bytes
        || format!("{:x}", Sha256::digest(&bytes)) != expected.sha256
    {
        return Err(RerankError::Provider(
            "local model asset checksum mismatch".into(),
        ));
    }
    Ok(bytes)
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
    fn asset_sizes_and_checksums_are_checked_without_native_model_loading() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model");
        let expected = Asset {
            bytes: 3,
            sha256: format!("{:x}", Sha256::digest(b"abc")),
        };
        assert!(read_asset(&path, &expected).is_err());
        std::fs::write(&path, b"ab").unwrap();
        assert!(read_asset(&path, &expected).is_err());
        std::fs::write(&path, b"abd").unwrap();
        assert!(read_asset(&path, &expected).is_err());
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(read_asset(&path, &expected).unwrap(), b"abc");
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
