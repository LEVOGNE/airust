#![doc = include_str!("../../docs/LOCAL-EMBEDDINGS.md")]

use super::{EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace};
use fastembed::{
    AutoDevicePolicy, InitOptionsUserDefined, Pooling, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use sha2::{Digest, Sha256};
use std::path::Path;

const MAX_MODEL_BYTES: u64 = 512 * 1024 * 1024;
const MAX_TOKENIZER_BYTES: u64 = 64 * 1024 * 1024;
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

pub struct FastEmbedProvider {
    space: EmbeddingSpace,
    model: TextEmbedding,
}

impl FastEmbedProvider {
    pub fn multilingual_e5_small(
        model_directory: impl AsRef<Path>,
        runtime_library: impl AsRef<Path>,
        threads: usize,
    ) -> Result<Self, EmbeddingError> {
        if threads == 0 {
            return Err(EmbeddingError::Provider(
                "thread count must be positive".into(),
            ));
        }
        let root = model_directory.as_ref();
        let mut fingerprint = Sha256::new();
        fingerprint.update(b"airust/e5-small/mean/max512/query: /passage: /v1");
        let mut read = |name: &str, maximum: u64| -> Result<Vec<u8>, EmbeddingError> {
            let bytes = crate::input::BoundedFile::open(&root.join(name), maximum)
                .and_then(|file| file.read())
                .map_err(provider_error)?;
            fingerprint.update((name.len() as u64).to_le_bytes());
            fingerprint.update(name.as_bytes());
            fingerprint.update((bytes.len() as u64).to_le_bytes());
            fingerprint.update(&bytes);
            Ok(bytes)
        };
        let model = read("onnx/model.onnx", MAX_MODEL_BYTES)?;
        let tokenizer = TokenizerFiles {
            tokenizer_file: read("tokenizer.json", MAX_TOKENIZER_BYTES)?,
            config_file: read("config.json", MAX_CONFIG_BYTES)?,
            special_tokens_map_file: read("special_tokens_map.json", MAX_CONFIG_BYTES)?,
            tokenizer_config_file: read("tokenizer_config.json", MAX_CONFIG_BYTES)?,
        };
        let space = EmbeddingSpace::new(
            format!(
                "intfloat/multilingual-e5-small:sha256:{:x}",
                fingerprint.finalize()
            ),
            384,
        )?;
        let environment =
            crate::local_runtime::environment(runtime_library.as_ref()).map_err(provider_error)?;
        let options = InitOptionsUserDefined::new()
            .with_environment(environment)
            .with_max_length(512)
            .with_intra_threads(threads)
            .with_auto_device(AutoDevicePolicy::PreferCPU);
        let model = TextEmbedding::try_new_from_user_defined(
            UserDefinedEmbeddingModel::new(model, tokenizer).with_pooling(Pooling::Mean),
            options,
        )
        .map_err(provider_error)?;
        Ok(Self { space, model })
    }
}

impl EmbeddingProvider for FastEmbedProvider {
    fn space(&self) -> &EmbeddingSpace {
        &self.space
    }

    fn embed(
        &mut self,
        texts: &[&str],
        purpose: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.iter().any(|text| text.trim().is_empty()) {
            return Err(EmbeddingError::EmptyInput);
        }
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let prefix = match purpose {
            EmbeddingPurpose::Query => "query: ",
            EmbeddingPurpose::Document => "passage: ",
        };
        let inputs: Vec<_> = texts.iter().map(|text| format!("{prefix}{text}")).collect();
        self.model.embed(inputs, Some(32)).map_err(provider_error)
    }
}

fn provider_error(error: impl std::fmt::Display) -> EmbeddingError {
    EmbeddingError::Provider(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_or_nonregular_assets_fail_before_native_initialization() {
        let dir = tempfile::tempdir().unwrap();
        let onnx = dir.path().join("onnx");
        std::fs::create_dir(&onnx).unwrap();
        let model = std::fs::File::create(onnx.join("model.onnx")).unwrap();
        model.set_len(MAX_MODEL_BYTES + 1).unwrap();
        let result = FastEmbedProvider::multilingual_e5_small(dir.path(), "missing-runtime", 2);
        assert!(
            matches!(result, Err(EmbeddingError::Provider(error)) if error.contains("byte limit"))
        );
        model.set_len(0).unwrap();
        let tokenizer = std::fs::File::create(dir.path().join("tokenizer.json")).unwrap();
        tokenizer.set_len(MAX_TOKENIZER_BYTES + 1).unwrap();
        let result = FastEmbedProvider::multilingual_e5_small(dir.path(), "missing-runtime", 2);
        assert!(
            matches!(result, Err(EmbeddingError::Provider(error)) if error.contains("byte limit"))
        );
        tokenizer.set_len(0).unwrap();
        std::fs::create_dir(dir.path().join("config.json")).unwrap();
        let result = FastEmbedProvider::multilingual_e5_small(dir.path(), "missing-runtime", 2);
        assert!(
            matches!(result, Err(EmbeddingError::Provider(error)) if error.contains("regular file"))
        );
    }
}
