# AIRust 0.1.8: Was geändert wurde und wozu es gut ist

Stand: 16. November 2026. Die Überarbeitung liegt auf dem lokalen Branch `codex/airust-v0.1.8`.

AIRust ist jetzt zuerst eine Wissensbibliothek. Sie findet Texte, vergleicht Inhalte und schlägt mögliche Verbindungen vor. Eine Anwendung kann sie direkt einbinden. Sie braucht dafür weder Dashboard noch Webserver noch Sprachmodell.

## Der neue Kern

| Änderung | Nutzen |
| --- | --- |
| `KnowledgeEntry` mit ID, Text und optionalen Metadaten | Derselbe Inhalt kann aus DenkMaps, einem Agenten oder einer anderen Rust-Anwendung kommen. |
| Ein gemeinsamer `Engine`-Index | Suche, Duplikate und Verbindungsvorschläge arbeiten mit demselben Bestand. |
| BM25 und ein eigener TF-IDF-Suchmodus | Anwendungen können passende Textstellen nach verschiedenen lexikalischen Verfahren ordnen. |
| `upsert()` und `remove()` nach ID | Einzelne Inhalte werden ersetzt oder entfernt, ohne den gesamten Bestand neu aufzubauen. |
| Einheitliche Unicode-Verarbeitung | Zahlen bleiben erhalten. Satzzeichen trennen Wörter. Umlaute werden als Zeichen gezählt. |
| Exakter Vergleich, Fuzzy-Vergleich und Wortüberschneidung | Anwendungen können gleiche Texte, Tippfehler oder ähnliche Wortmengen unterscheiden. |
| Duplikatgruppen | Doppelte Texte lassen sich vor dem Löschen mit ihren Metadaten prüfen. |
| `Context` für IDs und Metadaten | Die Suche lässt sich auf einen gewünschten Bereich begrenzen. |
| Erklärte `relate()`-Vorschläge | Jede mögliche Verbindung nennt die gemeinsamen Begriffe. Die Anwendung entscheidet über die Übernahme. |
| JSON-Import und JSON-Export | Die Anwendung kann Einträge speichern und den Suchindex daraus wiederherstellen. |

Die neue API gibt bei Nichttreffern eine leere Liste zurück. Fehlerhafte Einträge werden vor einer Änderung abgelehnt. Doppelte IDs beim Import ergeben einen Fehler. Gleiche Rangwerte werden nach ID geordnet.

Beziehungen sind in dieser Version lexikalische Vorschläge. Ein gemeinsames Wort beweist weder eine Voraussetzung noch einen Widerspruch. Für Bedeutungsvergleiche kann eine Anwendung später einen externen Anbieter ergänzen.

## Der Standard-Build ist kleiner

Vorher aktivierte der Standard-Build Dashboard, HTTP, SQLite und Bots. Außerdem war PDF-Verarbeitung immer dabei. Jetzt sind diese Teile optional.

Der normale Abhängigkeitsbaum enthält auf diesem Mac **23 statt 256 Pakete**, jeweils einschließlich AIRust. Der frühere Build ohne Standardfunktionen enthielt noch 120 Pakete. Die Zählung verwendet `cargo tree -e normal`, zählt jede Paketversion einmal und gilt für diesen Prüfstand.

Mit sämtlichen Erweiterungen sind es nach den Aktualisierungen 286 Pakete. Besonders die aktualisierte optionale PDF-Bibliothek bringt zusätzliche Verarbeitung mit. Diese Größe betrifft den Standardkern nicht.

Die alten Agentennamen, Trainingsformate und Programme bleiben verfügbar. Die nicht verwendeten direkten Abhängigkeiten für Farben, Diagramme und die bisherige IndexMap wurden entfernt. Die alten Funktionsnamen `colors` und `plotting` bleiben als leere Kompatibilitätsnamen erhalten.

Die Abhängigkeiten wurden über `cargo update` innerhalb der vorhandenen kompatiblen Versionsbereiche aktualisiert. Die Lockdatei hält den geprüften Stand fest. Die PDF-Integration verlangt mindestens die tatsächlich geprüfte Version `pdf_oxide 0.3.78`. Das Projekt verlangt Rust 1.89; diese Mindestversion wurde ebenfalls geprüft.

## Behobene Fehler

- Der alte BM25-Agent berechnete für jeden Eintrag erneut die mittlere Länge des gesamten Bestands. Er verwendet jetzt den gemeinsamen Index mit bereits erfassten Längen.
- Das Ergänzen von Trainingsbeispielen baut die bestehenden Suchverweise nicht mehr vollständig neu auf.
- Der Tokenizer klebt Wörter an Satzzeichen nicht mehr zusammen und entfernt Zahlen nicht mehr.
- Fuzzy-Grenzen verwenden Zeichen statt Byte-Längen. Die Ähnlichkeit ist auf die längere Zeichenfolge bezogen.
- Agenten geben konsistente Konfidenzwerte weiter. Ein gespeicherter Antworttext kann selbst „No matching answer found“ enthalten, ohne deshalb als Nichttreffer zu gelten.
- Agenten und der Gesprächsadapter geben Metadaten weiter. Die HTTP-Antwort kann sie ebenfalls enthalten.
- Alte und neue Antwortformate dürfen in derselben Trainingsdatei stehen. Metadaten bleiben dabei erhalten.
- PDF-Herkunftsdaten enthalten Datei, Seite und Abschnitt. Der Weg zur `KnowledgeBase` verwirft sie nicht mehr.
- PDF-Einstellungen werden geprüft. Ungültige Überlappungen führen zu einem verständlichen Fehler. Lange Unicode-Texte bleiben innerhalb der Abschnittsgrenze. Der letzte kurze Rest wird behalten.
- Wiederholt verwendete PDF-Regulärausdrücke werden nur einmal vorbereitet. Gültige Unicode-Zeichen werden nicht allein wegen ihrer hohen Zeichennummer als beschädigter Text gewertet.
- Die Dashboard-Datenbank ergänzt beim nächsten Start eine optionale Metadatenspalte. Bestehende Zeilen bleiben erhalten. Importierte Herkunftsdaten überleben einen Neustart.
- Trainingsausgaben des Dashboards lassen sich wieder importieren. Feldnamen der Ausgabe werden beim Import akzeptiert; JSON-Antworten behalten ihr Format.
- `merge_kb` liest seine eigene Ausgabe nicht mehr als Quelle ein. Es entfernt vollständig identische Beispiele. Eine fehlerhafte Quelle lässt eine vorhandene Ausgabe bestehen. Die Ausgabe wird über eine temporäre Datei ersetzt.
- `build.rs` schreibt nur im Build-Verzeichnis. Der Build legt keine Wissensdateien im Quellordner mehr an.
- Port null, fehlende Portangaben und Port 65535 werden abgefangen. Der zweite Port für die Konsole kann nicht mehr überlaufen.

## Werkzeuge und Texte

README und Startanleitung wurden neu geschrieben. Sie erklären zuerst den Zweck und danach die Benutzung. Die drei alten Sprachfassungen mit denselben langen Wiederholungen wurden durch eine kompakte deutsche Projektanleitung ersetzt. Die optionale Oberfläche und ihre Übersetzungen bleiben erhalten.

Die Beispieldaten enthalten weiterhin dieselben 124 Fragen. Ihre Angaben zu Produktkern, Version, Suchverfahren, Funktionen und Startbefehlen wurden korrigiert. Es wird nicht mehr behauptet, BM25 verstehe Bedeutung oder Synonyme.

Die Oberfläche erhält ihre Versionsanzeige aus der Paketversion. Veraltete mitgelieferte Übersetzungen werden nur ersetzt, wenn sie noch dem alten Standard entsprechen. Eigene Übersetzungen bleiben bestehen.

Der Webserver bindet standardmäßig an `127.0.0.1`. `AIRUST_BIND_ADDR` erlaubt eine ausdrücklich gewählte andere Adresse. Die pauschale CORS-Freigabe wurde entfernt. Die Konsole bleibt ein Werkzeug mit festgelegten Verwaltungsbefehlen; sie ist keine freie Shell.

Docker baut ausdrücklich die Web-Erweiterung und berücksichtigt beide Ports. Die Startanleitung bleibt im Build-Kontext, weil der Server sie einbettet. Datenbank, Sicherungsarchive und lokale Protokolle werden nicht in das Image übernommen. Das Rust-Basisimage folgt dem aktuellen stabilen Bookworm-Image.

Eine MIT-Lizenzdatei, eine Umstellungsanleitung, ein Architekturtext, ein Änderungsverzeichnis und eine Anleitung für Mitwirkende wurden ergänzt. `test.sh` führt echte Prüfungen aus. Eine GitHub-Actions-Datei automatisiert dieselben Schritte und die Prüfung der Mindestversion.

Alle Rust-Dateien wurden einheitlich formatiert. Bei einigen älteren Web-Dateien bestehen die Änderungen nur aus dieser Formatierung. Die Web-Vektorspeicher bleiben kompatible optionale Adapter mit ihren bisherigen Speicherformaten.

## Gemessener Laufzeitvergleich

Derselbe Beispielcode wurde gegen das gesicherte Original und gegen den neuen Stand gebaut. Gemessen wurden 1.000 Trainingsbeispiele, 100 Suchanfragen und das Ergänzen eines Beispiels. Die Anfrage enthält Begriffe, die in allen Beispielen vorkommen. Jede Version lief dreimal im Release-Modus auf diesem Mac. Die Tabelle zeigt den mittleren der drei gemessenen Werte, ohne Kompilierzeit.

| Vorgang | 0.1.7 | 0.1.8 |
| --- | ---: | ---: |
| Erster Indexaufbau | 0,794 ms | 1,711 ms |
| 100 Suchanfragen | 228,691 ms | 17,529 ms |
| Ein Beispiel ergänzen | 0,704 ms | 0,004 ms |

In diesem kleinen Fall ist die Suche etwa 13-mal schneller. Der erste Aufbau dauert ungefähr doppelt so lange, weil der neue Index zusätzliche Verweise und normalisierte Texte vorbereitet. Das Ergänzen profitiert davon, dass bestehende Einträge nicht erneut zerlegt werden.

Die Zeiten sind lokale Einzelmessungen. Der neue Tokenizer behält außerdem die Zahlen in den Beispielen bei. Der Vergleich misst deshalb das Verhalten der beiden vollständigen Versionen, keine isolierte mathematische Operation. Er belegt keine allgemeine Antwortzeit oder Kapazität für andere Datenmengen.

Wiederholen:

```bash
cargo run --locked --release --example benchmark -- 1000 100
```

## Durchgeführte Prüfungen

Geprüft auf macOS, ARM64, mit Rust/Cargo 1.98.0. Die Mindestversion wurde mit Rust 1.89.0 kompiliert.

| Prüfung | Ergebnis |
| --- | --- |
| Ausgangsstand 0.1.7 | 168 Tests bestanden; sechs Clippy-Warnungen |
| Standardbibliothek 0.1.8 | 180 Tests einschließlich README-Beispiel bestanden |
| Sämtliche Funktionen 0.1.8 | 200 Tests einschließlich CLI, PDF, Datenbankumstellung und README-Beispiel bestanden |
| Formatierung | `cargo fmt --all -- --check` erfolgreich |
| Clippy für Kern, alle Funktionen und CLI allein | Jeweils mit `-D warnings` erfolgreich |
| `pdf`, `bots` und `async` einzeln | Bibliotheks-Builds erfolgreich |
| API-Dokumentation | Mit `RUSTDOCFLAGS="-D warnings"` erfolgreich |
| Ausführbares Kernbeispiel | Erfolgreich, einschließlich Duplikaten, Filtern und JSON-Rückweg |
| Rust 1.89.0 | Alle Funktionen und alle Ziele mit `cargo check` erfolgreich |
| Lokaler HTTP-Test | Status, Dashboard, Anleitung, Anfrage und zweiter Konsolenport erfolgreich |
| Paketprüfung | Crate gebaut und überprüft; Datenbank und Sicherung werden ausgeschlossen |

Zu den neuen Verhaltenstests gehören eine unabhängig berechnete BM25-/TF-IDF-Referenz, der Vergleich von gezielten Indexänderungen mit einem frisch aufgebauten Index, Unicode-Fälle, Grenzwerte, Duplikate, Metadaten und das unveränderte Ziel bei fehlgeschlagenem Zusammenfügen.

Das vollständige lokale Protokoll des Prüflaufs liegt unter `/private/tmp/airust-validation-0.1.8.log`. Das Protokoll der Mindestversion liegt unter `/private/tmp/airust-msrv-1.89.log`.

## Sicherung und Stand der Veröffentlichung

Das Original liegt unter `backups/airust-v0.1.7-original.tar.gz` und als lokaler Ausgangscommit `fb7badb` auf `baseline/airust-v0.1.7`. Archivpfad und Prüfsumme stehen in der [Bestandsaufnahme](ANALYSE-0.1.7.md). Die vorhandene Datei `airust.db` ist gegenüber diesem Ausgangscommit unverändert.

Die Crate enthält Quellcode, Beispiele, Tests und Anleitungen. Sie enthält weder die lokale Datenbank noch das Sicherungsarchiv. Die Änderungen lassen sich über den Entwicklungsbranch und die Git-Historie nachvollziehen. Der gesicherte Ausgangsbaum ist identisch mit dem ursprünglichen GitHub-Commit `052510449fe99a047e69e2dd6781a369498eedd1`; die Version 0.1.8 baut auf dieser Historie auf.

Den Veröffentlichungsstand zeigen [GitHub Releases](https://github.com/LEVOGNE/airust/releases) und [crates.io](https://crates.io/crates/airust). Die Prüfergebnisse der GitHub-Läufe stehen unter [Actions](https://github.com/LEVOGNE/airust/actions).

DenkMaps und ein externer MCP-Agent wurden hier nicht geändert; die Bibliotheks-API und das Beispiel bereiten deren Einbindung vor. Docker war auf diesem Rechner nicht verfügbar; der Container-Build konnte deshalb nicht ausgeführt werden.
