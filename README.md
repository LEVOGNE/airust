# airust

🧠 **airust** ist eine modulare, trainierbare KI-Bibliothek in Rust.  
Sie unterstützt Compile-Zeit-Wissen über JSON-Dateien und erlaubt einfache Vorhersage-Engines für natürliche Spracheingaben.

## 🚀 Features

- 🧩 Modularer Aufbau mit `TrainableAgent`-Trait
- 🧠 Mehrere eingebaute Agenten:
  - `SimpleAgent` (exakte Übereinstimmung)
  - `FuzzyAgent` (Levenshtein-Ähnlichkeit)
  - `ContextAgent` (berücksichtigt Gesprächskontext)
  - `TfidfAgent` (nutzt BM25-Algorithmus für bessere Ähnlichkeitssuche)
  - `StructuredAgent` (unterstützt strukturierte Antwortformate)
- 💾 Compile-Zeit-Wissen via `knowledge/train.json`
- ⚖️ Gewichtete Trainingsdaten für präzisere Antworten
- 📋 Erweiterbare dynamische Wissensbasis zur Laufzeit
- 🔍 Erweiterte Texterkennung mit TF-IDF und BM25
- 🏷️ Unterstützung für strukturierte Antworten (Text, Markdown, JSON)
- 📦 Leicht in andere Projekte einbindbar
- 🖥️ CLI-Testprogramm inklusive

## 🔧 Verwendung

### In deinem Projekt

```toml
[dependencies]
airust = { path = "../airust" }
```

### Beispielcode

```rust
use airust::simple_agent::SimpleAgent;
use airust::knowledge::TRAINING_DATA;
use airust::agent::TrainableAgent;

fn main() {
    let mut ai = SimpleAgent::new();
    ai.train(&TRAINING_DATA);
    let antwort = ai.predict("Was ist airust?");
    println!("Antwort: {}", antwort);
}
```

## 📂 Trainingsdaten

Die Datei `knowledge/train.json` unterstützt nun auch Gewichtungen:

```json
[
  {
    "input": "Was ist GEL?",
    "output": "Ein leichtes Versionskontrollsystem.",
    "weight": 1.0
  },
  {
    "input": "Was ist airust?",
    "output": "Ein modularer KI-Agent in Rust.",
    "weight": 2.0
  }
]
```

Diese Datei wird automatisch bei Build-Zeit in das Binary eingebunden (`build.rs` kümmert sich darum).

## 🖥️ CLI-Nutzung

```bash
# Verschiedene Agenten testen
cargo run --bin cli -- simple "Was ist GEL?"
cargo run --bin cli -- fuzzy "Was ist Gel"
cargo run --bin cli -- tfidf "Erkläre mir airust"
cargo run --bin cli -- context "Folge-Frage zum Thema"
```

## 🧪 Testen der erweiterten Funktionen

### Kontext-Agent testen

```bash
# Interaktiven Kontext-Test starten
cargo run --bin context_test
```

Der Kontext-Agent speichert vorherige Fragen und Antworten, um bessere Ergebnisse bei zusammenhängenden Gesprächen zu liefern.

### Dynamische Wissensbasis

```bash
# Testen der dynamischen Wissensdatenbank
cargo run --bin knowledge_test
```

Mit der dynamischen Wissensbasis können Sie zur Laufzeit:

- Neue Trainingsdaten hinzufügen
- Die Wissensbasis speichern und laden
- Änderungen an Trainingsdaten vornehmen

### Strukturierte Antworten

Der `StructuredAgent` unterstützt verschiedene Antwortformate:

- Einfacher Text
- Markdown-formatierter Text
- JSON-strukturierte Daten

```bash
# Testen der strukturierten Antworten
cargo run --bin structured_test
```

## 📊 Fortgeschrittene Verwendung

### BM25-Algorithmus für bessere Trefferquoten

Der `TfidfAgent` verwendet den BM25-Algorithmus, eine Erweiterung des TF-IDF-Verfahrens, um die semantische Ähnlichkeit zwischen Fragen besser zu erkennen:

```rust
use airust::tfidf_agent::TfidfAgent;
use airust::knowledge::TRAINING_DATA;
use airust::agent::TrainableAgent;

fn main() {
    let mut ai = TfidfAgent::new();
    ai.train(&TRAINING_DATA);
    // Findet Antworten auch bei anders formulierten Fragen
    let antwort = ai.predict("Erkläre mir, was airust kann");
    println!("{}", antwort);
}
```

## 📃 Lizenz

MIT

---

> Entwickelt mit ❤️ in Rust.  
> Dieses Crate ist offen für Beiträge und Erweiterungen.
