# Von AIRust 0.1.7 auf 0.1.8 umsteigen

0.1.8 richtet AIRust auf die Nutzung als Bibliothek aus. Einige Voreinstellungen ändern sich bewusst. Prüfe diese Punkte, bevor du die Versionsangabe deiner Anwendung erhöhst.

## Für eine neue Anwendung

Verwende `Engine` und `KnowledgeEntry`. Jeder Eintrag hat eine ID, einen Text und optionale Metadaten. Übernimm vorhandene IDs aus deiner Anwendung. Derselbe Aufruf von `upsert()` mit derselben ID ersetzt den alten Eintrag.

Verwende `search()` für Suchbegriffe, `find_matches()` für Inhaltsvergleiche und `relate()` für erste Verbindungsvorschläge. Die Anwendung entscheidet über deren Speicherung.

`match` ist in Rust ein reserviertes Wort. Die gleichwertige Schreibweise lautet deshalb `r#match()`. `find_matches()` ist die lesbare Alternative.

## Für bestehende Bibliotheksnutzer

Die bisherigen Typen, Traits und Agentennamen bleiben erhalten. Für Dashboard und PDF müssen Funktionen jetzt ausdrücklich aktiviert werden:

```toml
# Bibliothek mit der bisherigen Web-Erweiterung:
[dependencies]
airust = { version = "0.1.8", features = ["web"] }
```

Benötigst du nur die PDF-Bibliotheksfunktionen, reicht `features = ["pdf"]`. Für die Programme `airust` und `merge_kb` benötigst du `cli`. `pdf2kb` benötigt `cli` und `pdf`. Ohne Funktionen wird ausschließlich die Bibliothek gebaut.

Die bisherigen Funktionsnamen `colors` und `plotting` werden weiterhin akzeptiert. Sie sind leer, weil der alte Quellcode diese Bibliotheken bereits nicht verwendete. `async` aktiviert weiterhin Tokio; die neue Kern-API ist synchron. `bots` aktiviert die vollständige bestehende Web-Integration.

## Beispielwissen ist ausdrücklich optional

`KnowledgeBase::from_embedded()` liefert ohne `embedded-knowledge` eine leere Sammlung. `cli` und `web` aktivieren das Beispielwissen automatisch. Der neue Wissensindex startet immer mit deinen eigenen Einträgen.

Der Build schreibt keine Dateien mehr in `knowledge/`. Er arbeitet nur im Cargo-Build-Verzeichnis.

## Textverarbeitung und Rangwerte ändern sich

Der gemeinsame Tokenizer behält Zahlen bei. Satzzeichen trennen Wörter, statt sie zusammenzukleben. Unicode-Schreibweisen werden vereinheitlicht. Dadurch können frühere Suchreihenfolgen anders ausfallen. Gespeicherte lexikalische Vektoren der optionalen Bot-Integration sollten neu aufgebaut werden.

Im neuen Index gilt NFKC plus Kleinschreibung und vereinheitlichte Leerzeichen für exakte Duplikate. Akzente und Satzzeichen bleiben bedeutend. Der bisherige `MatchAgent::new_exact()` behält seine strengere Behandlung von Leerzeichen bei.

Fuzzy-Abstände und Grenzwerte zählen Unicode-Zeichen statt UTF-8-Bytes. Die Ähnlichkeit verwendet die längere der beiden Zeichenfolgen als Bezugsgröße. Konfidenzwerte können deshalb von 0.1.7 abweichen.

`TfidfAgent` verwendet weiterhin BM25 und die bisherigen Beispielgewichte. Der neue Suchmodus `SearchAlgorithm::Tfidf` bietet tatsächlich TF-IDF mit Kosinusähnlichkeit. Die Agenten verwenden zur Kompatibilität weiterhin englische Antworttexte für Nichttreffer; die neue `Engine` gibt eine leere Trefferliste zurück.

BM25 ist kein semantisches Modell. Die alten `confidence`-Werte bleiben Heuristiken. Sie sind weder verlässlich kalibrierte Wahrscheinlichkeiten noch einheitliche Maße zwischen verschiedenen Agententypen.

## Trainingsdaten bleiben lesbar

`KnowledgeBase` liest alte Textantworten und neuere Antworten mit `Text`, `Markdown` oder `Json`. Beide Schreibweisen dürfen jetzt in derselben Datei vorkommen. Vorhandene Metadaten bleiben erhalten.

Das Trainingsformat `input`/`output` und das neue Eintragsformat `id`/`text` haben verschiedene Aufgaben. Ein automatischer Wechsel würde die Bedeutung von Frage und Antwort sowie die Vergabe dauerhafter IDs offenlassen. Eine Anwendung muss diese Zuordnung ausdrücklich vornehmen. Ein einfaches Beispiel ist: gespeicherte Datensatz-ID als `id`, Dokumentinhalt als `text` und Herkunft als `metadata`.

## PDF und Zusammenfügen

PDF-Einstellungen werden geprüft. Die Überlappung muss kleiner als die maximale Textlänge sein. `min_chunk_size` ist eine bevorzugte Mindestlänge. Ein letzter kurzer Rest wird nicht mehr verworfen. Metadaten enthalten die ursprüngliche Seitennummer; `chunk_index` und `total_chunks` beziehen sich auf die jeweilige Seite.

`merge_kb` fragt nicht mehr interaktiv nach Beispieldaten. Dafür gibt es `--include-embedded`. Die Ausgabe wird nicht erneut eingelesen. Alle Quellen müssen gültig sein, bevor eine vorhandene Zieldatei ersetzt wird.

## Dashboard und Docker

Der Server bindet jetzt standardmäßig an `127.0.0.1`. Mit `AIRUST_BIND_ADDR` kann ausdrücklich eine andere Adresse gewählt werden. Der zweite Port für die Konsole bleibt erforderlich. Die pauschale CORS-Freigabe wurde entfernt; eigene Web-Clients auf anderen Ursprüngen benötigen eine passende vorgeschaltete Konfiguration.

Die bestehende SQLite-Datei wurde während der Überarbeitung nicht geändert. Beim nächsten Start ergänzt der Server die Trainingsdatentabelle um eine optionale Metadatenspalte. Vorhandene Zeilen bleiben erhalten. PDF-Herkunftsdaten und importierte Metadaten überleben dadurch einen Neustart. Trainingsausgaben können wieder importiert werden; `output_text` und `output_format` werden als Feldnamen akzeptiert. Agentenantworten der HTTP-API können zusätzlich `metadata` enthalten.

Veraltete mitgelieferte Oberflächentexte werden beim nächsten Start nur dann korrigiert, wenn sie noch dem alten Standard entsprechen. Selbst geänderte Übersetzungen bleiben erhalten.

Das Docker-Image baut `--features web` und verwendet ein aktuelles stabiles Rust-Image statt Rust 1.85. Die aktuelle Auflösung der optionalen Abhängigkeiten benötigt mindestens Rust 1.89. Die Lockdatei hält die geprüften Paketversionen fest.

## Was bleibt Aufgabe der Anwendung?

DenkMaps speichert und zeichnet seine Knoten selbst. Ein MCP-Agent bietet seine eigenen Werkzeuge und seinen Transport an. AIRust liefert durchsuchbare Inhalte und Vergleichsergebnisse. Eine Integration in diese anderen Projekte wurde hier vorbereitet und als Bibliotheksbeispiel gezeigt, aber nicht in fremden Repositories implementiert.
