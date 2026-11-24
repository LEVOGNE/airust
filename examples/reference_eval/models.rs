use super::Result;
use airust::{
    reranking::Reranker,
    semantic::{Embedding, EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace},
};
use serde::Deserialize;
use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};

#[derive(Deserialize)]
pub struct Vectors {
    pub dataset_identity: String,
    pub corpus_sha256: String,
    pub recipe: String,
    pub dimensions: usize,
    pub documents: BTreeMap<String, Vec<f32>>,
    pub queries: BTreeMap<String, Vec<f32>>,
}

impl Vectors {
    pub fn load(path: &Path, identity: &str, corpus: &str) -> Result<Self> {
        let value: Self = serde_json::from_reader(BufReader::new(File::open(path)?))?;
        if value.dataset_identity != identity || value.corpus_sha256 != corpus {
            return Err("vectors do not match the frozen corpus identity".into());
        }
        Ok(value)
    }

    pub fn space(&self) -> Result<EmbeddingSpace> {
        Ok(EmbeddingSpace::new(
            format!("{};airust-l2-f32", self.recipe),
            self.dimensions,
        )?)
    }
}

pub struct QueryProvider {
    pub space: EmbeddingSpace,
    pub queries: BTreeMap<String, Vec<f32>>,
}

impl EmbeddingProvider for QueryProvider {
    fn space(&self) -> &EmbeddingSpace {
        &self.space
    }
    fn embed(
        &mut self,
        texts: &[&str],
        _purpose: EmbeddingPurpose,
    ) -> std::result::Result<Vec<Vec<f32>>, EmbeddingError> {
        texts
            .iter()
            .map(|text| {
                self.queries.get(*text).cloned().ok_or_else(|| {
                    EmbeddingError::Provider(
                        "query vector missing from the frozen preparation".into(),
                    )
                })
            })
            .collect()
    }
}

pub struct Dense {
    pub space: EmbeddingSpace,
    pub documents: BTreeMap<String, Embedding>,
    pub queries: BTreeMap<String, Vec<f32>>,
}

impl Dense {
    pub fn from_vectors(vectors: Vectors) -> Result<Self> {
        let space = vectors.space()?;
        let documents = vectors
            .documents
            .into_iter()
            .map(|(id, values)| Ok((id, Embedding::new(space.clone(), values)?)))
            .collect::<Result<_>>()?;
        Ok(Self {
            space,
            documents,
            queries: vectors.queries,
        })
    }

    pub fn query(&self, query: &str) -> Result<Embedding> {
        Ok(Embedding::new(
            self.space.clone(),
            self.queries
                .get(query)
                .ok_or("query vector is missing")?
                .clone(),
        )?)
    }
}

pub fn independent_cosine(left: &[f32], right: &[f32]) -> f64 {
    let mut dot = 0.0;
    let mut left_squared = 0.0;
    let mut right_squared = 0.0;
    for (&a, &b) in left.iter().zip(right) {
        let a = f64::from(a);
        let b = f64::from(b);
        dot += a * b;
        left_squared += a * a;
        right_squared += b * b;
    }
    (dot / left_squared.sqrt() / right_squared.sqrt()).clamp(-1.0, 1.0)
}

pub fn native_reranker(directory: &Path, runtime: &Path) -> Result<Box<dyn Reranker>> {
    #[cfg(feature = "local-reranking")]
    {
        Ok(Box::new(
            airust::reranking::local::FastEmbedReranker::multilingual_minilm_v2(
                directory, runtime, 2,
            )?,
        ))
    }
    #[cfg(not(feature = "local-reranking"))]
    {
        let _ = (directory, runtime);
        Err("enable local-reranking for the explicit native-model profile".into())
    }
}
