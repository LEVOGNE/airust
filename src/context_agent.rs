// src/context_agent.rs - Revised ContextAgent
use crate::agent::{Agent, ContextualAgent, ResponseFormat, TrainableAgent, TrainingExample};
use std::collections::VecDeque;

/// Context agent wraps another agent and provides context-aware responses
pub struct ContextAgent<A: Agent> {
    base_agent: A,
    context_history: VecDeque<(String, ResponseFormat)>, // (Question, Answer)
    max_context_items: usize,
    context_format: ContextFormat,
}

/// Configurable context formatting strategies
pub enum ContextFormat {
    /// Format: "Q: question A: answer Q: question A: answer ..."
    QAPairs,
    /// Format: "[question -> answer, question -> answer, ...]"
    List,
    /// Format: "Previous questions and answers: question - answer; question - answer; ..."
    Sentence,
    /// Custom format with formatting function
    Custom(Box<dyn Fn(&[(String, ResponseFormat)]) -> String>),
}

impl Default for ContextFormat {
    fn default() -> Self {
        ContextFormat::QAPairs
    }
}

impl<A: Agent> ContextAgent<A> {
    /// Creates a new context agent with a base agent and maximum context items
    pub fn new(base_agent: A, max_context_items: usize) -> Self {
        Self {
            base_agent,
            context_history: VecDeque::new(),
            max_context_items,
            context_format: ContextFormat::default(),
        }
    }

    /// Sets the context format for generating context strings
    pub fn with_context_format(mut self, format: ContextFormat) -> Self {
        self.context_format = format;
        self
    }

    /// Creates a context string from the conversation history
    fn get_context_string(&self) -> String {
        match &self.context_format {
            ContextFormat::QAPairs => {
                let mut context = String::new();
                for (q, a) in &self.context_history {
                    let answer_text: String = a.clone().into();
                    context.push_str(&format!("Q: {} A: {} ", q, answer_text));
                }
                context
            }
            ContextFormat::List => {
                let items: Vec<String> = self
                    .context_history
                    .iter()
                    .map(|(q, a)| {
                        let answer_text: String = a.clone().into();
                        format!("{} -> {}", q, answer_text)
                    })
                    .collect();
                format!("[{}]", items.join(", "))
            }
            ContextFormat::Sentence => {
                let items: Vec<String> = self
                    .context_history
                    .iter()
                    .map(|(q, a)| {
                        let answer_text: String = a.clone().into();
                        format!("{} - {}", q, answer_text)
                    })
                    .collect();
                format!("Previous questions and answers: {}", items.join("; "))
            }
            ContextFormat::Custom(formatter) => formatter(
                &self
                    .context_history
                    .iter()
                    .map(|(q, a)| (q.clone(), a.clone()))
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl<A: Agent> Agent for ContextAgent<A> {
    /// Generates a response with context added to the input
    fn predict(&self, input: &str) -> ResponseFormat {
        // Adds context to input
        let context_str = self.get_context_string();
        let enhanced_input = if context_str.is_empty() {
            input.to_string()
        } else {
            format!("{} [Context: {}]", input, context_str)
        };

        self.base_agent.predict(&enhanced_input)
    }
}

impl<A: TrainableAgent> TrainableAgent for ContextAgent<A> {
    /// Trains the base agent with the provided training data
    fn train(&mut self, data: &[TrainingExample]) {
        self.base_agent.train(data);
    }
}

impl<A: Agent> ContextualAgent for ContextAgent<A> {
    /// Adds a new context item to the conversation history
    fn add_context(&mut self, question: String, answer: ResponseFormat) {
        self.context_history.push_back((question, answer));

        // Keeps size under maximum
        while self.context_history.len() > self.max_context_items {
            self.context_history.pop_front();
        }
    }

    /// Clears the entire context history
    fn clear_context(&mut self) {
        self.context_history.clear();
    }
}
