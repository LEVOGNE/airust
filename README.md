# airust

🧠 **airust** is a modular, trainable AI library written in Rust.  
It supports compile-time knowledge through JSON files and allows for simple prediction engines for natural language input.

---

## 🚀 Features

- 🧩 **Modular architecture with unified traits:**

  - `Agent` – Base trait for all agents
  - `TrainableAgent` – For trainable agents
  - `ContextualAgent` – For context-aware agents

- 🧠 **Advanced agent implementations:**

  - `MatchAgent` – Handles both exact and fuzzy matches (replaces `SimpleAgent` and `FuzzyAgent`)
  - `TfidfAgent` – Uses the BM25 algorithm for improved similarity detection
  - `ContextAgent<A>` – Generic wrapper for context-aware conversations

- 📝 **Unified response format:**

  - `ResponseFormat` – Supports responses in plain text, Markdown, and JSON

- 💾 **Enhanced knowledge base:**

  - Compile-time knowledge via `knowledge/train.json`
  - Runtime knowledge expansion supported
  - Backward compatibility with older data formats

- 🛠️ **Simplified CLI tool:**
  - Single CLI tool for all operations
  - Interactive mode for testing and experimentation
  - Knowledge base management capabilities

---

## 🔧 Usage

### Integration in other projects

```toml
[dependencies]
airust = "0.1.4"
```

### Sample Code (Updated)

```rust
use airust::{Agent, TrainableAgent, MatchAgent, ResponseFormat, KnowledgeBase};

fn main() {
    // Load embedded knowledge base
    let kb = KnowledgeBase::from_embedded();

    // Create and train agent
    let mut agent = MatchAgent::new_exact();
    agent.train(kb.get_examples());

    // Ask a question
    let answer = agent.predict("What is airust?");

    // Print the response (converted from ResponseFormat to String)
    println!("Answer: {}", String::from(answer));
}
```

---

## 📂 Training Data Format

The file format `knowledge/train.json` has been extended to support both the old and new format:

```json
[
  {
    "input": "What is airust?",
    "output": {
      "Text": "A modular AI library in Rust."
    },
    "weight": 2.0
  },
  {
    "input": "What agents are available?",
    "output": {
      "Markdown": "- **MatchAgent** (exact & fuzzy)\n- **TfidfAgent** (BM25)\n- **ContextAgent** (context-aware)"
    },
    "weight": 1.0
  }
]
```

Legacy format is still supported for backward compatibility.

---

## 🖥️ CLI Usage

```bash
# Simple query
airust query simple "What is airust?"
airust query fuzzy "What is airust?"
airust query tfidf "Explain airust"

# Interactive mode
airust interactive

# Knowledge base management
airust knowledge
```

---

## 📊 Advanced Usage – Context Agent

```rust
use airust::{Agent, TrainableAgent, ContextualAgent, TfidfAgent, ContextAgent, KnowledgeBase};

fn main() {
    // Load embedded knowledge base
    let kb = KnowledgeBase::from_embedded();

    // Create and train base agent
    let mut base_agent = TfidfAgent::new();
    base_agent.train(kb.get_examples());

    // Wrap in a context-aware agent (remembering 3 turns)
    let mut agent = ContextAgent::new(base_agent, 3);

    // First question
    let answer1 = agent.predict("What is airust?");
    println!("A1: {}", String::from(answer1.clone()));

    // Add to context history
    agent.add_context("What is airust?".to_string(), answer1);

    // Follow-up question
    let answer2 = agent.predict("What features does it provide?");
    println!("A2: {}", String::from(answer2));
}
```

---

## 📃 License

MIT

> Built with ❤️ in Rust.  
> Contributions and extensions are welcome!

---

## 🛠 Migration Guide for airust 0.1.4

This guide helps you migrate from airust 0.1.x to 0.1.4.

### 1. Trait and Type Changes

#### New Trait Hierarchy

```rust
trait Agent {
    fn predict(&self, input: &str) -> ResponseFormat;
}

trait TrainableAgent: Agent {
    fn train(&mut self, data: &[TrainingExample]);
}

trait ContextualAgent: Agent {
    fn add_context(&mut self, question: String, answer: ResponseFormat);
}
```

#### New Response Format

```rust
let answer: ResponseFormat = agent.predict("Question");
let answer_string: String = String::from(answer);
```

#### Updated TrainingExample Struct

```rust
struct TrainingExample {
    input: String,
    output: ResponseFormat,
    weight: f32,
}
```

---

### 2. Agent Replacements

#### SimpleAgent and FuzzyAgent → MatchAgent

```rust
let mut agent = MatchAgent::new_exact();
let mut agent = MatchAgent::new_fuzzy();
```

With options:

```rust
let mut agent = MatchAgent::new(MatchingStrategy::Fuzzy(FuzzyOptions {
    max_distance: Some(5),
    threshold_factor: Some(0.2),
}));
```

#### ContextAgent is Now Generic

```rust
let mut base_agent = TfidfAgent::new();
base_agent.train(&data);
let mut agent = ContextAgent::new(base_agent, 5);
```

#### StructuredAgent Removed (use ResponseFormat)

---

### 3. Knowledge Base Changes

```rust
let kb = KnowledgeBase::from_embedded();
let data = kb.get_examples();

let mut kb = KnowledgeBase::new();
kb.add_example("Question".to_string(), "Answer".to_string(), 1.0);
```

---

### 4. CLI Tool Migration

```bash
cargo run --bin airust -- query simple "What is airust?"
cargo run --bin airust -- interactive
cargo run --bin airust -- knowledge
```

---

### 5. Recommendations

- Upgrade your dependencies
- Use new `lib.rs` re-exports
- Test thoroughly
- Explore new context formatting
