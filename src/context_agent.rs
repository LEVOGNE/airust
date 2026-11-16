// src/context_agent.rs - Revised ContextAgent
use crate::agent::{
    Agent, ContextualAgent, PredictionResult, ResponseFormat, TrainableAgent, TrainingExample,
};
use std::collections::VecDeque;

/// Context agent wraps another agent and provides context-aware responses
pub struct ContextAgent<A: Agent> {
    base_agent: A,
    context_history: VecDeque<(String, ResponseFormat)>, // (Question, Answer)
    max_context_items: usize,
    context_format: ContextFormat,
}

type ContextFormatFn = Box<dyn Fn(&[(String, ResponseFormat)]) -> String + Send + Sync>;

/// Configurable context formatting strategies
#[derive(Default)]
pub enum ContextFormat {
    /// Format: "Q: question A: answer Q: question A: answer ..."
    #[default]
    QAPairs,
    /// Format: "[question -> answer, question -> answer, ...]"
    List,
    /// Format: "Previous questions and answers: question - answer; question - answer; ..."
    Sentence,
    /// Custom format with formatting function
    Custom(ContextFormatFn),
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

    fn enhanced_input(&self, input: &str) -> String {
        let context = self.get_context_string();
        if context.is_empty() {
            input.to_owned()
        } else {
            format!("{} [Context: {}]", input, context)
        }
    }
}

impl<A: Agent> Agent for ContextAgent<A> {
    /// Generates a response with context added to the input
    fn predict(&self, input: &str) -> ResponseFormat {
        self.base_agent.predict(&self.enhanced_input(input))
    }

    fn predict_with_metadata(&self, input: &str) -> PredictionResult {
        self.base_agent
            .predict_with_metadata(&self.enhanced_input(input))
    }

    fn confidence(&self, input: &str) -> f32 {
        self.base_agent.confidence(&self.enhanced_input(input))
    }
}

impl<A: TrainableAgent> TrainableAgent for ContextAgent<A> {
    /// Trains the base agent with the provided training data (replaces all)
    fn train(&mut self, data: &[TrainingExample]) {
        self.base_agent.train(data);
    }

    /// Appends training data to the base agent without replacing
    fn append(&mut self, data: &[TrainingExample]) {
        self.base_agent.append(data);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_agent::MatchAgent;
    use crate::tfidf_agent::TfidfAgent;

    fn make_example(input: &str, output: &str) -> TrainingExample {
        TrainingExample {
            input: input.to_string(),
            output: ResponseFormat::Text(output.to_string()),
            weight: 1.0,
            metadata: None,
        }
    }

    fn trained_match_agent() -> MatchAgent {
        let mut agent = MatchAgent::new_exact();
        agent.train(&[
            make_example("hello", "Hi there!"),
            make_example("goodbye", "See you later!"),
        ]);
        agent
    }

    // === No-Context Passthrough ===

    #[test]
    fn test_no_context_passthrough() {
        let base = trained_match_agent();
        let ctx = ContextAgent::new(base, 5);
        // Without context, input goes through unchanged
        assert_eq!(String::from(ctx.predict("hello")), "Hi there!");
    }

    #[test]
    fn test_context_modifies_input() {
        let base = trained_match_agent();
        let mut ctx = ContextAgent::new(base, 5);
        ctx.add_context("prev".to_string(), ResponseFormat::Text("ans".to_string()));
        // With context, the input gets "[Context: ...]" appended
        // so exact match on "hello" should fail
        let result = String::from(ctx.predict("hello"));
        assert!(result.contains("No matching answer found"));
    }

    // === FIFO Eviction ===

    #[test]
    fn test_fifo_eviction() {
        let base = trained_match_agent();
        let mut ctx = ContextAgent::new(base, 2);

        ctx.add_context("q1".to_string(), ResponseFormat::Text("a1".to_string()));
        ctx.add_context("q2".to_string(), ResponseFormat::Text("a2".to_string()));
        ctx.add_context("q3".to_string(), ResponseFormat::Text("a3".to_string()));

        // Only q2 and q3 should remain (FIFO: q1 evicted)
        let context_str = ctx.get_context_string();
        assert!(!context_str.contains("q1"));
        assert!(context_str.contains("q2"));
        assert!(context_str.contains("q3"));
    }

    #[test]
    fn test_clear_context() {
        let base = trained_match_agent();
        let mut ctx = ContextAgent::new(base, 5);
        ctx.add_context("q1".to_string(), ResponseFormat::Text("a1".to_string()));
        ctx.clear_context();

        let context_str = ctx.get_context_string();
        assert!(context_str.is_empty());
        // After clearing, passthrough should work again
        assert_eq!(String::from(ctx.predict("hello")), "Hi there!");
    }

    #[test]
    fn test_add_text_context() {
        let base = trained_match_agent();
        let mut ctx = ContextAgent::new(base, 5);
        ctx.add_text_context("question".to_string(), "answer".to_string());

        let context_str = ctx.get_context_string();
        assert!(context_str.contains("question"));
        assert!(context_str.contains("answer"));
    }

    // === Context Format Tests ===

    #[test]
    fn test_format_qa_pairs() {
        let base = trained_match_agent();
        let mut ctx = ContextAgent::new(base, 5).with_context_format(ContextFormat::QAPairs);
        ctx.add_context("q1".to_string(), ResponseFormat::Text("a1".to_string()));

        let context_str = ctx.get_context_string();
        assert!(context_str.contains("Q: q1 A: a1"));
    }

    #[test]
    fn test_format_list() {
        let base = trained_match_agent();
        let mut ctx = ContextAgent::new(base, 5).with_context_format(ContextFormat::List);
        ctx.add_context("q1".to_string(), ResponseFormat::Text("a1".to_string()));

        let context_str = ctx.get_context_string();
        assert!(context_str.starts_with('['));
        assert!(context_str.ends_with(']'));
        assert!(context_str.contains("q1 -> a1"));
    }

    #[test]
    fn test_format_sentence() {
        let base = trained_match_agent();
        let mut ctx = ContextAgent::new(base, 5).with_context_format(ContextFormat::Sentence);
        ctx.add_context("q1".to_string(), ResponseFormat::Text("a1".to_string()));

        let context_str = ctx.get_context_string();
        assert!(context_str.starts_with("Previous questions and answers:"));
        assert!(context_str.contains("q1 - a1"));
    }

    #[test]
    fn test_format_custom() {
        let base = trained_match_agent();
        let mut ctx = ContextAgent::new(base, 5).with_context_format(ContextFormat::Custom(
            Box::new(|items| {
                items
                    .iter()
                    .map(|(q, a)| format!("{}={}", q, String::from(a.clone())))
                    .collect::<Vec<_>>()
                    .join("|")
            }),
        ));
        ctx.add_context("q1".to_string(), ResponseFormat::Text("a1".to_string()));
        ctx.add_context("q2".to_string(), ResponseFormat::Text("a2".to_string()));

        let context_str = ctx.get_context_string();
        assert_eq!(context_str, "q1=a1|q2=a2");
    }

    #[test]
    fn test_format_multiple_items() {
        let base = trained_match_agent();
        let mut ctx = ContextAgent::new(base, 5).with_context_format(ContextFormat::List);
        ctx.add_context("q1".to_string(), ResponseFormat::Text("a1".to_string()));
        ctx.add_context("q2".to_string(), ResponseFormat::Text("a2".to_string()));

        let context_str = ctx.get_context_string();
        assert!(context_str.contains("q1 -> a1"));
        assert!(context_str.contains("q2 -> a2"));
    }

    // === Edge Cases ===

    #[test]
    fn test_max_context_items_zero() {
        let base = trained_match_agent();
        let mut ctx = ContextAgent::new(base, 0);
        ctx.add_context("q1".to_string(), ResponseFormat::Text("a1".to_string()));

        // With max 0, all context is evicted immediately
        let context_str = ctx.get_context_string();
        assert!(context_str.is_empty());
    }

    // === Train Delegation ===

    #[test]
    fn test_train_delegates_to_base() {
        let base = MatchAgent::new_exact();
        let mut ctx = ContextAgent::new(base, 5);
        ctx.train(&[make_example("hello", "Hi!")]);
        // Without context, the base agent should respond
        assert_eq!(String::from(ctx.predict("hello")), "Hi!");
    }

    #[test]
    fn test_append_delegates_to_base() {
        let base = MatchAgent::new_exact();
        let mut ctx = ContextAgent::new(base, 5);
        ctx.train(&[make_example("hello", "Hi!")]);
        ctx.append(&[make_example("bye", "Goodbye!")]);
        assert_eq!(String::from(ctx.predict("hello")), "Hi!");
        assert_eq!(String::from(ctx.predict("bye")), "Goodbye!");
    }

    // === ContextAgent wrapping TfidfAgent ===

    #[test]
    fn test_context_agent_with_tfidf() {
        let mut base = TfidfAgent::new();
        base.train(&[
            make_example("rust programming", "Rust is fast"),
            make_example("python scripting", "Python is easy"),
        ]);
        let ctx = ContextAgent::new(base, 5);
        let result = String::from(ctx.predict("rust"));
        assert!(result.contains("Rust"));
    }
}
