# Änderungen

## 0.1.8 – Bibliothekskern – 16. November 2026

- Allgemeine Wissenseinträge mit IDs, Text und Metadaten.
- Gemeinsamer, gezielt aktualisierbarer Suchindex für BM25 und TF-IDF.
- Einheitlicher Textvergleich, Duplikatgruppen und erklärte Beziehungsvorschläge.
- Explizite Suchfilter statt unbemerkter Änderung der Anfrage.
- Standardmäßig keine PDF-, HTTP-, Datenbank- oder Bot-Abhängigkeit.
- Bestehende Agenten und Trainingsformate als Adapter erhalten.
- Unicode-Grenzen, Konfidenzweitergabe, Metadaten und PDF-Textstücke korrigiert.
- Zusammenfügen von Trainingsdateien ohne wiederholtes Einlesen der Ausgabe.
- Abhängigkeiten innerhalb ihrer kompatiblen Versionsbereiche aktualisiert.
- Dokumentation, Startbefehle, Dashboard-Texte, Docker-Konfiguration und Tests überarbeitet.

Details: [Änderungsbericht](docs/AENDERUNGEN-0.1.8.md), [Umstellung](docs/MIGRATION-0.1.8.md).

Quellcode und Versionspakete: [GitHub](https://github.com/LEVOGNE/airust), [crates.io](https://crates.io/crates/airust).

## 0.1.7 – Gesicherter Ausgangsstand

Der gelieferte Ordner enthielt die bisherigen Agenten, PDF-Import, CLI, Dashboard, SQLite und Bots. Vor der Überarbeitung bestanden 158 Modultests und zehn Integrationstests. Die [Bestandsaufnahme](docs/ANALYSE-0.1.7.md) beschreibt diesen Stand.
