use crate::agent::{TrainableAgent, TrainingExample};
use crate::fuzzy_agent::FuzzyAgent;
use std::collections::VecDeque;

pub struct ContextAgent {
    base_agent: FuzzyAgent,
    context_history: VecDeque<(String, String)>, // (Frage, Antwort)
    max_context_items: usize,
}

impl ContextAgent {
    pub fn new(max_context_items: usize) -> Self {
        Self {
            base_agent: FuzzyAgent::new(),
            context_history: VecDeque::new(),
            max_context_items,
        }
    }

    pub fn add_to_context(&mut self, question: String, answer: String) {
        self.context_history.push_back((question, answer));

        // Halte die Größe unter dem Maximum
        while self.context_history.len() > self.max_context_items {
            self.context_history.pop_front();
        }
    }

    // Erzeugt einen Kontext-String aus der Historie
    fn get_context_string(&self) -> String {
        let mut context = String::new();
        for (q, a) in &self.context_history {
            context.push_str(&format!("F: {} A: {} ", q, a));
        }
        context
    }
}

impl TrainableAgent for ContextAgent {
    fn train(&mut self, data: &[TrainingExample]) {
        self.base_agent.train(data);
    }

    fn predict(&self, input: &str) -> String {
        // Kontext zur Eingabe hinzufügen
        let context_str = self.get_context_string();
        let enhanced_input = if context_str.is_empty() {
            input.to_string()
        } else {
            format!("{} [Kontext: {}]", input, context_str)
        };

        let answer = self.base_agent.predict(&enhanced_input);

        // Hier können wir zusätzliche Logik hinzufügen, um die Antwort
        // basierend auf dem Kontext zu modifizieren
        answer
    }
}
