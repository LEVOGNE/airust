# Technische Bestandsaufnahme von AIRust 0.1.7

Stand: 16. November 2026. Dieser Bericht beschreibt den gelieferten Ordner vor der Überarbeitung.

## Was wurde gesichert und geprüft?

Der Ordner enthielt ein Rust-Paket namens `airust`, Version 0.1.7. Es gab keine `.git`-Historie. Deshalb wurde zuerst eine vollständige Archivkopie angelegt. Anschließend wurde der unveränderte Stand lokal als Commit `fb7badb` auf `baseline/airust-v0.1.7` gesichert. Die Entwicklung erfolgt auf `codex/airust-v0.1.8`. Die frühere GitHub-Historie wurde damit nicht wiederhergestellt.

Das Archiv liegt unter `/private/tmp/airust-v0.1.7-original-20261116-015240.tar.gz`.
Seine SHA-256-Prüfsumme lautet `56355ca8cd6d92363da7808829f6aa4b3466598711378b3eb41cd46666b45d4d`.
Eine zusätzliche Kopie wird im Projekt unter `backups/` aufbewahrt. Dieser Ordner wird nicht mit der Crate ausgeliefert.

Geprüft mit Rust und Cargo 1.98.0 auf macOS:

| Prüfung | Ergebnis vor der Änderung |
| --- | --- |
| `cargo metadata --no-deps` | Ein Paket, drei Programme, eine Integrationstestsammlung |
| `cargo test --workspace --locked --offline` | 158 Modultests und 10 Integrationstests bestanden |
| `cargo test --workspace --no-default-features --locked --offline` | Dieselben 168 Tests bestanden |
| `cargo clippy --workspace --all-targets --locked --offline` | Erfolgreich, aber sechs Warnungen |

Ein bestandener Test beweist nur das geprüfte Verhalten. Die bisherigen Tests prüfen vor allem Agenten und Hilfsfunktionen. Sie belegen keine allgemeine Antwortqualität, keine semantische Suche und keine vollständige Prüfung des Webservers.

## Welche Algorithmen existieren tatsächlich?

| Datei oder Bereich | Vorhandene Funktion | Fachliche Einordnung |
| --- | --- | --- |
| `src/agent.rs::text_utils` | Kleinschreibung, Tokenisierung, Stoppwörter, N-Gramme, Levenshtein, Jaccard | Klassische Textverarbeitung |
| `src/match_agent.rs` | Exakter Vergleich und Levenshtein-Abstand | Erkennt gleiche Schreibweisen und Tippfehler |
| `src/tfidf_agent.rs` | BM25 mit Beispielgewichten | Lexikalische Suche; trotz des Namens kein eigener TF-IDF-Suchmodus |
| `src/context_agent.rs` | Begrenzter Gesprächsverlauf; Anhängen an die Anfrage | Textgedächtnis; kein Verstehen von Folgefragen |
| `src/knowledge.rs` | JSON lesen, schreiben, zusammenfügen; eingebettete Trainingsbeispiele | Frage-Antwort-Sammlung ohne stabile Eintrags-IDs |
| `src/pdf_loader.rs` | PDF-Textextraktion, Reinigung und Textstücke | Importwerkzeug; keine lernende Engine |
| `src/web/vectordb.rs` | TF-IDF-Vektoren und Kosinusähnlichkeit | Lexikalische Vektoren; keine neuronalen Embeddings |
| `src/web/bots/vectordb.rs` | Weitere TF-IDF-Berechnung und Datenbanksuche | Zweiter Suchweg mit anderem Tokenizer |
| `src/web/bots/processor.rs` | Vorlagen für Fragen, Absätze, Sätze und Inhalts-Hashes | Regelbasierte Aufbereitung und Duplikatprüfung |
| `src/web/bots/crawler.rs` | HTTP-Abfragen und HTML-Auswertung | Datenbeschaffung |

Es gibt keine eigene LLM-Engine, kein neuronales Training und keinen belegten Synonymvergleich. Aussagen wie „BM25 versteht Bedeutung und Synonyme“ in der alten Dokumentation und Oberfläche sind zu weitgehend.

## Welche Abhängigkeiten gibt es?

```text
agent: Typen, Traits und Textfunktionen
  ├── match_agent
  ├── tfidf_agent
  ├── context_agent: umhüllt einen anderen Agenten
  └── knowledge: speichert TrainingExample
        └── pdf_loader: erzeugt Trainingsbeispiele

CLI und Web verwenden diese Agenten und die KnowledgeBase.
Web ergänzt SQLite, HTTP, Konsole, Bots und zwei weitere Suchwege.
```

Standardmäßig waren `web` und `colors` eingeschaltet. `web` aktivierte zusätzlich Bots, Tokio, SQLite und HTTP-Bibliotheken. `pdf_oxide`, `chrono`, `regex`, `once_cell` und `indexmap` waren auch ohne Standardfunktionen direkte Abhängigkeiten. `colored`, `plotly` und `plotters` wurden im Quellcode nicht verwendet.

## Wo liegen konkrete Probleme?

1. **Die BM25-Suche macht unnötige Arbeit.** Für jedes Dokument wird erneut die mittlere Länge aller Dokumente ausgerechnet. Dadurch entsteht zusätzlich quadratischer Aufwand. Jede Suche bewertet außerdem sämtliche Dokumente.
2. **Ergänzungen bauen den alten Suchindex vollständig neu.** Einzelne Änderungen und Löschungen nach stabiler ID fehlen.
3. **Die Textzerlegung ist uneinheitlich.** Der zentrale Tokenizer entfernt Zahlen und Satzzeichen. Aus `Rust-Python` wird `rustpython`. Die Bot-Suche zerlegt denselben Text anders.
4. **Fuzzy-Grenzen verwenden Byte-Längen.** Der Levenshtein-Abstand zählt Zeichen. Bei Umlauten passen diese Einheiten nicht zusammen.
5. **Die API vermischt Treffer und Antworttexte.** Ein Nichttreffer wird als englischer Satz zurückgegeben. Der Standard-Konfidenzwert untersucht diesen Antworttext. Die detaillierte Konfidenz und der normale Agentenaufruf können voneinander abweichen.
6. **Metadaten gehen verloren.** PDF-Beispiele enthalten Herkunftsdaten. Der Weg über `pdf_to_knowledge_base()` verwirft diese wieder. Suchergebnisse der alten Agenten reichen Metadaten ebenfalls nicht weiter.
7. **Der Build kann Quelldateien verändern.** Fehlt `knowledge/train.json`, erzeugt das alte `build.rs` die Datei im Projekt. Bibliotheksnutzer sollten beim Kompilieren keine neuen Wissensdateien bekommen.
8. **PDF-Grenzwerte sind ungeprüft.** Ein zu großer Überlappungswert kann eine Subtraktion unterlaufen lassen. Lange Textsegmente können die konfigurierte Maximalgröße überschreiten.
9. **Zusammenfügen kann Daten vervielfachen.** Das alte `merge_kb` liest auch seine eigene Zieldatei `train.json` wieder ein und kann eingebettete Daten erneut hinzufügen.
10. **Werkzeuge und Anleitung passen nicht überall zusammen.** `test.sh` verwendet veraltete CLI-Aufrufe. Die Versionsnummer steht mehrfach fest im Webbereich. Eine Lizenzdatei fehlt trotz MIT-Angabe.

Die Weboberfläche bietet Datei- und Datenbankzugriff. Ihre Konsole kennt Verwaltungsbefehle einschließlich Datenbank-Reset. Entgegen der alten README führt sie keine freien Shell-Befehle aus. Der bestehende Server ist daher ein Verwaltungswerkzeug. Seine Beibehaltung als optionale Integration ist keine Aussage über eine sichere öffentliche Bereitstellung.

## Was kann erhalten und herausgelöst werden?

BM25, Levenshtein und Jaccard sind brauchbare Grundlagen. Textverarbeitung und Ranking lassen sich gemeinsam verwenden. Dafür braucht es weder HTTP noch SQLite, PDF oder ein Sprachmodell.

Die bisherigen Agentennamen und Trainingsformate können als Kompatibilitätsschicht erhalten bleiben. PDF, CLI und Dashboard werden über Cargo-Funktionen zugeschaltet. Die vorhandene Datenbank und die vorhandenen Wissensdateien bleiben erhalten.

## Was fehlt für DenkMaps und andere Anwendungen?

- Allgemeine Einträge mit stabiler ID, Text und optionalen Metadaten.
- Treffer mit Eintragsbezug, Rangwert und nachvollziehbaren Suchbegriffen.
- Gezieltes Einfügen, Ersetzen und Entfernen im Index.
- Ein gemeinsamer Unicode-Tokenizer mit Zahlen und sauberen Wortgrenzen.
- Duplikatgruppen und ein einheitlicher Inhaltsvergleich.
- Suchfilter für ausgewählte IDs oder Metadaten, ohne den Fragetext zu verändern.
- Beziehungsvorschläge mit sichtbarer Begründung. Eine Wortüberschneidung darf keinen Widerspruch oder eine Voraussetzung behaupten.
- Ein kleines, ausführbares Integrationsbeispiel und Tests für Änderungen am Index.

Ein DenkMaps-Repository oder dessen Datenmodell liegt hier nicht vor. Eine tatsächliche DenkMaps-Integration kann deshalb erst im aufrufenden Projekt erfolgen. Auch ein MCP-Transport ist eine Aufgabe des aufrufenden Agenten.

## Architekturentscheidung nach dieser Analyse

AIRust 0.1.8 wird eine modulare Wissensbibliothek. `text`, `matching`, `search`, `context`, `knowledge` und `index` bilden den Kern. Ein gemeinsamer Index stellt `search()`, `find_matches()` beziehungsweise `r#match()` und einfache `relate()`-Vorschläge bereit.

`search()` und `match()` haben Vorrang. `relate()` verwendet dieselben Textdaten und schlägt nur lexikalisch verwandte Einträge vor. Fachliche Beziehungen wie „Voraussetzung“, „Unterthema“ oder „Widerspruch“ werden nicht automatisch abgeleitet. Die Anwendung entscheidet über Speicherung und Darstellung.

Die konkrete Umsetzung und ihre Prüfergebnisse stehen in [Änderungen 0.1.8](AENDERUNGEN-0.1.8.md). Die Umstellung für bestehende Nutzer steht in [Migration](MIGRATION-0.1.8.md).

Technische Grundlagen: [Cargo-Funktionen](https://doc.rust-lang.org/cargo/reference/features.html), [Unicode-Normalisierung](https://docs.rs/unicode-normalization/latest/unicode_normalization/trait.UnicodeNormalization.html). Die Aussagen über AIRust stammen aus dem lokalen Quellcode und den oben genannten Prüfungen.
