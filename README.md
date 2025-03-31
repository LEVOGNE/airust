# airust

🧠 **airust** is a modular, trainable AI library written in Rust.  
It supports compile-time knowledge through JSON files and provides sophisticated prediction engines for natural language input.

---

# 🚀 AiRust Capabilities

## ✅ **What You Can Concretely Do:**

### 🧠 1. **Build Your Own AI Agents**

- Train agents with examples (Question → Answer)
- Supported Agent Types:
  - **Exact Match** – precise matching
  - **Fuzzy Match** – tolerant to typos (Levenshtein)
  - **TF-IDF/BM25** – semantic similarity
  - **ContextAgent** – remembers previous dialogues

### 💬 2. **Manage Your Own Knowledge Database**

- Save/load training data (`train.json`)
- Weighting and metadata per entry
- Import legacy data possible

### 🧪 3. **Text Analysis**

- Tokenization, stop words, N-grams
- Similarity measures: Levenshtein, Jaccard
- Text normalization

### 🧰 4. **Custom CLI Tools**

- Launch `airust` CLI for:
  - Interactive sessions with an agent
  - Knowledge base management
  - Quick data testing

### 🌐 5. **Integration into Other Projects**

- Use `airust` as a Rust library in your own applications (Web, CLI, Desktop, IoT)

## 🔧 Example Application Ideas:

- 🤖 FAQ Bot for your website
- 📚 Intelligent document search
- 🧾 Customer support via terminal
- 🗣️ Voice assistant with context understanding
- 🔎 Similarity search for text databases
- 🛠 Local assistance tool for developer documentation

---

## 🚀 Advanced Features

- 🧩 **Modular Architecture with Unified Traits:**

  - `Agent` – Base trait for all agents with enhanced prediction capabilities
  - `TrainableAgent` – For agents that can be trained with examples
  - `ContextualAgent` – For context-aware conversational agents
  - `ConfidenceAgent` – New trait for agents that can provide prediction confidence

- 🧠 **Intelligent Agent Implementations:**

  - `MatchAgent` – Advanced matching with configurable strategies
    - Exact matching
    - Fuzzy matching with dynamic thresholds
    - Configurable Levenshtein distance options
  - `TfidfAgent` – Sophisticated similarity detection using BM25 algorithm
    - Customizable term frequency scaling
    - Document length normalization
  - `ContextAgent<A>` – Flexible context-aware wrapper
    - Multiple context formatting strategies
    - Configurable context history size

- 📝 **Enhanced Response Handling:**

  - `ResponseFormat` with support for:
    - Plain text
    - Markdown
    - JSON
  - Metadata and confidence tracking
  - Seamless type conversions

- 💾 **Intelligent Knowledge Base:**

  - Compile-time knowledge via `train.json`
  - Runtime knowledge expansion
  - Backward compatibility with legacy formats
  - Weighted training examples
  - Optional metadata support

- 🔍 **Advanced Text Processing:**

  - Tokenization with Unicode support
  - Stopword removal
  - Text normalization
  - N-gram generation
  - Advanced string similarity metrics
    - Levenshtein distance
    - Jaccard similarity

- 🛠️ **Unified CLI Tool:**
  - Interactive mode
  - Multiple agent type selection
  - Knowledge base management
  - Flexible querying

---

## 🔧 Usage

### Integration in other projects

```toml
[dependencies]
airust = "0.1.5"
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
    let mut base_agent = TfidfAgent::new()
        .with_bm25_params(1.5, 0.8);  // Custom BM25 tuning
    base_agent.train(kb.get_examples());

    // Wrap in a context-aware agent (remembering 3 turns)
    let mut agent = ContextAgent::new(base_agent, 3)
        .with_context_format(ContextFormat::List);

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

## 🚀 New in Version 0.1.5

### Matching Strategies

```rust
// Configurable fuzzy matching
let agent = MatchAgent::new(MatchingStrategy::Fuzzy(FuzzyOptions {
    max_distance: Some(5),      // Maximum Levenshtein distance
    threshold_factor: Some(0.2) // Dynamic length-based threshold
}));
```

### Context Formatting

```rust
// Multiple context representation strategies
let context_agent = ContextAgent::new(base_agent, 3)
    .with_context_format(ContextFormat::List);
    // Other formats: QAPairs, Sentence, Custom
```

### Advanced Text Utilities

```rust
// Text processing capabilities
let tokens = text_utils::tokenize("Hello, world!");
let unique_terms = text_utils::unique_terms(text);
let ngrams = text_utils::create_ngrams(text, 2);
```

---

## 📃 License

MIT

> Built with ❤️ in Rust.  
> Contributions and extensions are welcome!

---

## 🛠 Migration Guide for airust 0.1.5

This guide helps you migrate from airust 0.1.x to 0.1.5.

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
