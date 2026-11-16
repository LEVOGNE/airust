use crate::agent::{Agent, PredictionResult, ResponseFormat, TrainableAgent, TrainingExample};
use crate::context_agent::ContextAgent;
use crate::knowledge::KnowledgeBase;
use crate::match_agent::MatchAgent;
use crate::tfidf_agent::TfidfAgent;
use crate::web::bots::scheduler::BotScheduler;
use crate::web::db::Database;
use std::sync::Arc;
use tokio::sync::RwLock;

pub enum AgentWrapper {
    Exact(MatchAgent),
    Fuzzy(MatchAgent),
    Tfidf(TfidfAgent),
    Context(ContextAgent<TfidfAgent>),
}

impl AgentWrapper {
    pub fn predict(&self, input: &str) -> ResponseFormat {
        match self {
            AgentWrapper::Exact(a) => a.predict(input),
            AgentWrapper::Fuzzy(a) => a.predict(input),
            AgentWrapper::Tfidf(a) => a.predict(input),
            AgentWrapper::Context(a) => a.predict(input),
        }
    }

    pub fn predict_with_metadata(&self, input: &str) -> PredictionResult {
        match self {
            AgentWrapper::Exact(a) => a.predict_with_metadata(input),
            AgentWrapper::Fuzzy(a) => a.predict_with_metadata(input),
            AgentWrapper::Tfidf(a) => a.predict_with_metadata(input),
            AgentWrapper::Context(a) => a.predict_with_metadata(input),
        }
    }

    pub fn train(&mut self, data: &[TrainingExample]) {
        match self {
            AgentWrapper::Exact(a) => a.train(data),
            AgentWrapper::Fuzzy(a) => a.train(data),
            AgentWrapper::Tfidf(a) => a.train(data),
            AgentWrapper::Context(a) => a.train(data),
        }
    }

    pub fn add_context(&mut self, question: String, answer: ResponseFormat) {
        if let AgentWrapper::Context(a) = self {
            use crate::agent::ContextualAgent;
            a.add_context(question, answer);
        }
    }

    pub fn name(&self) -> &str {
        match self {
            AgentWrapper::Exact(_) => "exact",
            AgentWrapper::Fuzzy(_) => "fuzzy",
            AgentWrapper::Tfidf(_) => "tfidf",
            AgentWrapper::Context(_) => "context",
        }
    }
}

pub struct AppState {
    pub agents: Vec<AgentWrapper>,
    pub active_agents: Vec<String>,
    pub knowledge_base: KnowledgeBase,
    pub db: Arc<Database>,
    pub scheduler: BotScheduler,
    pub show_landing: bool,
}

impl AppState {
    pub fn query_best(&self, input: &str) -> (PredictionResult, String) {
        let mut best_result: Option<PredictionResult> = None;
        let mut best_agent = String::new();

        for agent in &self.agents {
            if self.active_agents.contains(&agent.name().to_string()) {
                let result = agent.predict_with_metadata(input);
                let dominated = best_result
                    .as_ref()
                    .is_some_and(|b| result.confidence <= b.confidence);
                if !dominated {
                    best_agent = agent.name().to_string();
                    best_result = Some(result);
                }
            }
        }

        if best_result.is_none() {
            if let Some(agent) = self.agents.first() {
                let result = agent.predict_with_metadata(input);
                best_agent = agent.name().to_string();
                best_result = Some(result);
            }
        }

        match best_result {
            Some(result) => (result, best_agent),
            None => (
                PredictionResult {
                    response: ResponseFormat::Text("No agents available".to_string()),
                    confidence: 0.0,
                    metadata: None,
                },
                "none".to_string(),
            ),
        }
    }

    pub fn train_all(&mut self, data: &[TrainingExample]) {
        for agent in &mut self.agents {
            agent.train(data);
        }
    }

    pub fn add_context_to_agents(&mut self, question: String, answer: ResponseFormat) {
        for agent in &mut self.agents {
            if self.active_agents.contains(&agent.name().to_string()) {
                agent.add_context(question.clone(), answer.clone());
            }
        }
    }

    pub fn active_agents_display(&self) -> String {
        self.active_agents.join(", ")
    }
}

pub type SharedState = Arc<RwLock<AppState>>;
