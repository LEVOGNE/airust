# airust

🧠 **airust** is a modular, trainable AI library in Rust.  
It supports compile-time knowledge through JSON files and enables simple prediction engines for natural language inputs.

## 🚀 Features

- 🧩 Modular architecture with the `TrainableAgent` trait
- 🧠 Multiple built-in agents:
  - `SimpleAgent` (exact matching)
  - `FuzzyAgent` (Levenshtein similarity)
  - `ContextAgent` (considers conversation context)
  - `TfidfAgent` (uses BM25 algorithm for better similarity matching)
  - `StructuredAgent` (supports structured response formats)
- 💾 Compile-time knowledge via `knowledge/train.json`
- ⚖️ Weighted training data for more precise answers
- 📋 Extensible knowledge base at runtime
- 🔍 Advanced text recognition with TF-IDF and BM25
- 🏷️ Support for structured responses (Text, Markdown, JSON)
- 📦 Easy integration into other projects
- 🖥️ CLI test program included

## 🔧 Usage

### In your project

```toml
[dependencies]
airust = { path = "../airust" }
```

### Example code

```rust
use airust::simple_agent::SimpleAgent;
use airust::knowledge::TRAINING_DATA;
use airust::agent::TrainableAgent;

fn main() {
    let mut ai = SimpleAgent::new();
    ai.train(&TRAINING_DATA);
    let answer = ai.predict("What is airust?");
    println!("Answer: {}", answer);
}
```

## 📂 Training Data

The file `knowledge/train.json` now also supports weights:

```json
[
  {
    "input": "What is GEL?",
    "output": "A lightweight version control system.",
    "weight": 1.0
  },
  {
    "input": "What is airust?",
    "output": "A modular AI agent in Rust.",
    "weight": 2.0
  }
]
```

This file is automatically embedded in the binary at build time (`build.rs` takes care of this).

## 🖥️ CLI Usage

```bash
# Testing different agents
cargo run --bin cli -- simple "What is GEL?"
cargo run --bin cli -- fuzzy "What is Gel"
cargo run --bin cli -- tfidf "Explain airust to me"
cargo run --bin cli -- context "Follow-up question on the topic"
```

## 🧪 Testing the Extended Features

### Context Agent Testing

```bash
# Start the interactive context test
cargo run --bin context_test
```

The Context Agent stores previous questions and answers to deliver better results in connected conversations.

### Dynamic Knowledge Base

```bash
# Test the dynamic knowledge database
cargo run --bin knowledge_test
```

With the dynamic knowledge base, you can at runtime:

- Add new training data
- Save and load the knowledge base
- Make changes to training data

### Structured Responses

The `StructuredAgent` supports different response formats:

- Simple text
- Markdown formatted text
- JSON structured data

```bash
# Test structured responses
cargo run --bin structured_test
```

## 📊 Advanced Usage

### BM25 Algorithm for Better Match Rates

The `TfidfAgent` uses the BM25 algorithm, an extension of the TF-IDF method, to better recognize semantic similarity between questions:

```rust
use airust::tfidf_agent::TfidfAgent;
use airust::knowledge::TRAINING_DATA;
use airust::agent::TrainableAgent;

fn main() {
    let mut ai = TfidfAgent::new();
    ai.train(&TRAINING_DATA);
    // Finds answers even with differently phrased questions
    let answer = ai.predict("Explain to me what airust can do");
    println!("{}", answer);
}
```

## 📃 License

MIT

---

> Developed with ❤️ in Rust.  
> This crate is open for contributions and extensions.
