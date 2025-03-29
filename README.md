# airust

🧠 **airust** ist eine modulare, trainierbare KI-Bibliothek in Rust.  
Sie unterstützt Compile-Zeit-Wissen über JSON-Dateien und erlaubt einfache Vorhersage-Engines für natürliche Spracheingaben.

## 🚀 Features

- 🧩 Modularer Aufbau mit `TrainableAgent`-Trait
- 🧠 Zwei eingebaute Agenten:
  - `SimpleAgent` (exakte Übereinstimmung)
  - `FuzzyAgent` (Levenshtein-Ähnlichkeit)
- 💾 Compile-Zeit-Wissen via `knowledge/train.json`
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

Die Datei `knowledge/train.json` sollte so aufgebaut sein:

```json
[
  { "input": "Was ist GEL?", "output": "Ein leichtes Versionskontrollsystem." },
  { "input": "Was ist airust?", "output": "Ein modularer KI-Agent in Rust." }
]
```

Diese Datei wird automatisch bei Build-Zeit in das Binary eingebunden (`build.rs` kümmert sich darum).

## 🖥️ CLI-Nutzung

```bash
cargo run --bin cli -- simple "Was ist GEL?"
cargo run --bin cli -- fuzzy "Was ist Gel"
```

## 📃 Lizenz

MIT

---

> Entwickelt mit ❤️ in Rust.  
> Dieses Crate ist offen für Beiträge und Erweiterungen.