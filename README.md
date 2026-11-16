# AIRust 0.1.8

**A Modular Knowledge Intelligence Engine**

AIRust ist eine kleine Wissensbibliothek für Rust. Sie findet passende Texte, erkennt ähnliche Inhalte und schlägt mögliche Verbindungen vor. Anwendungen wie DenkMaps oder MCP-Agenten können sie direkt verwenden.

Du gibst AIRust Einträge mit einer ID, einem Text und optionalen Zusatzdaten. AIRust liefert Ergebnisse zurück. Deine Anwendung entscheidet, wie sie diese anzeigt und speichert.

## Die drei Aufgaben

| Aufgabe | Was passiert? | Wozu ist das gut? |
| --- | --- | --- |
| `search()` | Findet Texte mit passenden Suchbegriffen. BM25 ordnet die Treffer. TF-IDF ist als weiterer Suchmodus verfügbar. | Notizen, Dokumentabschnitte und Wissen schnell wiederfinden. |
| `find_matches()` | Vergleicht Schreibweisen und Wörter. Erkennt gleiche Texte und Tippfehler. In Rust geht auch `r#match()`. | Doppelte oder ähnliche Einträge entdecken. |
| `relate()` | Schlägt Einträge mit gemeinsamen Begriffen vor. Jeder Vorschlag nennt diese Begriffe. | Mögliche Verbindungen prüfen und anschließend selbst übernehmen. |

BM25 und Textvergleich arbeiten mit Wörtern und Zeichen. Sie erkennen nicht automatisch, dass „Auto anmelden“ und „Kfz-Zulassung durchführen“ dasselbe bedeuten. Dafür kann eine Anwendung später ein Sprachmodell oder semantische Embeddings ergänzen. AIRust selbst benötigt kein LLM.

`relate()` erkennt in dieser Version keine Voraussetzungen, Unterthemen oder Widersprüche. Es speichert auch keine Beziehungen als Tatsachen.

## Direkt ausprobieren

Du brauchst Rust und Cargo. Das Projekt verlangt mindestens Rust 1.89. Der lokale Prüfstand verwendet Rust 1.98.0.

Im Projektordner:

```bash
cargo run --locked --example core
```

Das Beispiel zeigt Suche, ähnliche Texte, Duplikate, Verbindungsvorschläge, Filter und das Speichern als JSON. Es startet keinen Webserver und benötigt keine Zugangsdaten.

## In deine Anwendung einbauen

Füge AIRust als Abhängigkeit in deine `Cargo.toml` ein:

```toml
[dependencies]
airust = "0.1.8"
```

Für die Arbeit am Quellcode kannst du stattdessen den lokalen Ordner mit `airust = { path = "../airust-master" }` einbinden. Versionen und Downloads findest du auf [crates.io](https://crates.io/crates/airust).

Ein vollständiges Beispiel:

```rust
use airust::{Engine, KnowledgeEntry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wissen = Engine::new();
    wissen.upsert(KnowledgeEntry::new(
        "notiz-1",
        "Rust schützt den Speicher ohne Garbage Collector.",
    ))?;
    wissen.upsert(KnowledgeEntry::new(
        "notiz-2",
        "Python eignet sich für Skripte.",
    ))?;

    let treffer = wissen.search("Rust Speicher", 5);
    assert_eq!(treffer[0].entry.id, "notiz-1");
    println!("{}", treffer[0].entry.text);

    let aehnlich = wissen.find_matches(
        "Rust schützt den Speicher ohne Garbage Collector.",
        5,
    );
    assert_eq!(aehnlich[0].score, 1.0);

    // Dieselbe ID ersetzt einen Eintrag. Der Index wird dabei aktualisiert.
    wissen.upsert(KnowledgeEntry::new("notiz-2", "Python hilft bei Automatisierung."))?;
    wissen.remove("notiz-2");

    // JSON enthält die Einträge. Der Suchindex wird beim Laden neu aufgebaut.
    let geladen = Engine::from_json(&wissen.to_json()?)?;
    assert_eq!(geladen.len(), 1);
    Ok(())
}
```

Die IDs gehören deiner Anwendung. Verwende zum Beispiel bestehende Knoten-IDs aus DenkMaps. AIRust kennt weder Mindmaps noch deren Darstellung.

Suchtreffer enthalten den ursprünglichen Eintrag, seinen Rangwert und die passenden Begriffe. Kein Treffer ergibt eine leere Liste. BM25-Werte sind Rangwerte, keine Wahrscheinlichkeiten. Bei gleichem Rang entscheidet die ID, damit die Reihenfolge nachvollziehbar bleibt.

## Der Kern bleibt klein

Standardmäßig wird nur die Bibliothek gebaut. PDF, Webserver, Datenbank, Bots und Beispielwissen sind ausgeschaltet.

| Cargo-Funktion | Schaltet ein |
| --- | --- |
| Keine | Wissensindex, BM25, TF-IDF, Textvergleich, Duplikate, Filter und Beziehungsvorschläge |
| `cli` | Die bisherigen Kommandozeilenprogramme und eingebettetes Beispielwissen |
| `pdf` | PDF-Textextraktion und Aufteilung in Textstücke |
| `web` | Das bisherige Dashboard mit HTTP, SQLite, Bots, PDF und CLI |
| `bots` | Kompatibler Name für die Web- und Bot-Integration |
| `embedded-knowledge` | Die Beispieldaten aus `knowledge/train.json` beim Kompilieren einbetten |

Die alten Agenten `MatchAgent`, `TfidfAgent`, `ContextAgent` und `KnowledgeBase` bleiben verfügbar. `TfidfAgent` verwendet aus Kompatibilitätsgründen weiterhin diesen Namen; intern verwendet er die gemeinsame BM25-Suche.

## Die bisherigen Werkzeuge starten

```bash
# Eine Frage mit den bisherigen Trainingsbeispielen beantworten
cargo run --locked --features cli --bin airust -- cli query tfidf "Was ist AIRust?"

# Eine PDF-Datei in Trainingsdaten umwandeln
cargo run --locked --features cli,pdf --bin pdf2kb -- data/pdf/sample.pdf /tmp/airust-sample.json

# Das optionale Dashboard starten
cargo run --locked --features web --bin airust
```

Das Dashboard öffnest du unter `http://127.0.0.1:7070`. Seine Konsole verwendet Port 7071. Es bindet standardmäßig nur an den eigenen Rechner. Die Web-Erweiterung enthält Verwaltungsfunktionen für Dateien und Datenbank. Mehr zu Start, Docker und Netzwerk steht in [STARTUP.md](STARTUP.md).

## Was wurde verbessert?

- Ein gemeinsamer Index ersetzt die getrennte BM25-Implementierung. Eine Ergänzung baut den vorhandenen Index nicht mehr vollständig neu auf.
- Die Suche bewertet nur Einträge mit passenden Begriffen. Dokumentlängen werden beim Einfügen erfasst.
- Die Textverarbeitung behandelt Unicode, Zahlen und Wortgrenzen einheitlich. Umlaute zählen beim Vergleich als Zeichen.
- Einträge lassen sich über ihre ID gezielt ersetzen und löschen. Duplikatgruppen werden mit aktualisiert.
- Herkunftsdaten bleiben bei PDF-Importen und Agentenantworten erhalten. PDF-Textstücke überschreiten ihre Grenze nicht mehr; der letzte kurze Rest bleibt erhalten.
- Das Zusammenfügen von Wissensdateien ist wiederholbar. Eine fehlerhafte Quelle lässt die vorhandene Zieldatei bestehen.
- Anleitung, Versionsanzeigen, Startbefehle und Paketinhalt sind auf den Bibliothekskern ausgerichtet.

Der genaue Vorher-Nachher-Bericht steht unter [Änderungen 0.1.8](docs/AENDERUNGEN-0.1.8.md). Die [Bestandsaufnahme](docs/ANALYSE-0.1.7.md) beschreibt den Ausgangsstand und seine Grenzen. Für bestehende Anwendungen gibt es eine [Umstellungsanleitung](docs/MIGRATION-0.1.8.md).

## Was liegt wo?

| Ort | Inhalt |
| --- | --- |
| `src/text.rs` | Gemeinsame Textzerlegung und Normalisierung |
| `src/matching.rs` | Zeichenabstand und Wortüberschneidung |
| `src/search.rs` | Suchmodi, BM25-Einstellungen und Treffer |
| `src/context.rs` | Suchbereich nach IDs und Zusatzdaten einschränken |
| `src/knowledge.rs` | Allgemeine Wissenseinträge und die bisherige Trainingssammlung |
| `src/index.rs` | Gemeinsamer Index mit den drei Kernaufgaben |
| `src/*_agent.rs`, `src/agent.rs` | Kompatibilität mit der bisherigen Agenten-API |
| `src/pdf_loader.rs`, `src/bin/`, `src/web/` | Optionale Import- und Bedienwerkzeuge |
| `examples/` | Ausführbares Beispiel und ein einfacher Laufzeitvergleich |
| `tests/` | Tests für Kern, Integrationen und behobene Fehler |
| `docs/` | Analyse, Architektur, Umstellung und Änderungsbericht |
| `knowledge/`, `data/pdf/` | Vorhandene Beispieldaten |
| `airust.db` | Vorhandene lokale Dashboard-Datenbank; kein Bestandteil der veröffentlichten Crate |
| `backups/` | Gesichertes Original 0.1.7; kein Bestandteil der Crate |
| `build.rs` | Kopiert optionales Beispielwissen nur in das Build-Verzeichnis |
| `Dockerfile`, `.dockerignore` | Bauanleitung für das optionale Dashboard |
| `test.sh`, `.github/workflows/ci.yml` | Wiederholbare Qualitätsprüfungen |

Technische Details und weitere API-Beispiele stehen in der [Architekturbeschreibung](docs/ARCHITEKTUR.md). [CHANGELOG.md](CHANGELOG.md) fasst die Versionen zusammen.

## Selbst prüfen

```bash
./test.sh
cargo doc --locked --no-deps --open
```

Das Prüfskript kontrolliert Formatierung, Kern, optionale Funktionen, Dokumentationsbeispiele und Clippy. Die Bibliothek darf dabei keinen Webserver benötigen.

## Lizenz

MIT. Siehe [LICENSE](./LICENSE).
