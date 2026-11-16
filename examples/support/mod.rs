use airust::KnowledgeEntry;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Reference {
    pub version: String,
    pub documents: Vec<KnowledgeEntry>,
    pub queries: Vec<Query>,
}

#[derive(Deserialize)]
pub struct Query {
    pub id: String,
    pub text: String,
    pub relevant: Vec<String>,
}

pub fn reference() -> Result<Reference, serde_json::Error> {
    serde_json::from_str(include_str!("../../tests/fixtures/semantic-reference.json"))
}
