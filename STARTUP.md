# AIRust 0.1.8 starten

AIRust ist zuerst eine Rust-Bibliothek. Der Webserver ist eine optionale Erweiterung. Führe die Befehle in diesem Projektordner aus. Verwende Rust 1.89 oder neuer.

## 1. Den Kern ausprobieren

```bash
cargo run --locked --example core
```

Du siehst Suchtreffer, ähnliche Texte und mögliche Verbindungen. Es werden keine vorhandenen Wissensdateien oder Datenbanken verändert.

## 2. Die bisherigen Agenten verwenden

```bash
cargo run --locked --features cli --bin airust -- --version
cargo run --locked --features cli --bin airust -- cli query simple "Was ist AIRust?"
cargo run --locked --features cli --bin airust -- cli query fuzzy "Was ist AIRust?"
cargo run --locked --features cli --bin airust -- cli query tfidf "Was ist AIRust?"
cargo run --locked --features cli --bin airust -- cli interactive
```

`simple` vergleicht exakt, `fuzzy` toleriert Tippfehler und `tfidf` verwendet BM25. Die Agenten geben gespeicherte Antworten aus. Sie erzeugen keine neuen Antworten mit einem Sprachmodell.

`cli` bettet die vorhandenen Beispiele aus `knowledge/train.json` ein. Ändert sich diese Datei, berücksichtigt Cargo sie beim nächsten Build. Ohne `embedded-knowledge` enthält die Bibliothek keine eingebauten Beispieldaten.

## 3. Eine PDF importieren

```bash
cargo run --locked --features cli,pdf --bin pdf2kb -- data/pdf/sample.pdf /tmp/airust-sample.json
```

Das schreibt eine neue Trainingsdatei nach `/tmp`. Jede Textstelle erhält Herkunftsdaten mit Datei, Seite und Abschnitt. PDF-Dateien mit reinen Bildern benötigen eine separate Texterkennung. AIRust enthält keine OCR.

Weitere Einstellungen:

```bash
cargo run --locked --features cli,pdf --bin pdf2kb -- --help
```

Die maximale Textlänge muss größer als null sein. Der bevorzugte Mindestwert darf nicht größer als der Maximalwert sein. Die Überlappung muss kleiner als die maximale Textlänge sein. Ein letzter kurzer Textrest bleibt erhalten.

## 4. Trainingsdateien zusammenfügen

```bash
cargo run --locked --features cli --bin merge_kb -- quelle-a.json quelle-b.json --output /tmp/airust-gesamt.json
```

Das Werkzeug liest zuerst alle Quellen. Erst danach schreibt es die Ausgabe. Vollständig identische Beispiele werden nur einmal übernommen. Unterschiedliche Metadaten, Antworten oder Gewichte bleiben erhalten.

Ohne Dateiangaben verwendet es `knowledge/*.json` und schreibt `knowledge/train.json`. Die Zieldatei wird dabei nicht wieder als Quelle eingelesen. Gibt es keine Quellen oder ist eine Quelle fehlerhaft, bleibt eine vorhandene Ausgabe bestehen. `--include-embedded` fügt auf ausdrücklichen Wunsch die eingebetteten Beispiele hinzu.

Das Werkzeug verarbeitet das bisherige Format mit `input` und `output`. Das neue Format mit `id` und `text` lädt eine Anwendung mit `Engine::from_json()`.

## 5. Das Dashboard starten

```bash
cargo run --locked --features web --bin airust
```

Öffne `http://127.0.0.1:7070`. Die Konsole verwendet `ws://127.0.0.1:7071/ws/console`.

```bash
# Anderer Port: Die Konsole verwendet dann Port 8081.
cargo run --locked --features web --bin airust -- --port 8080

# Im Hintergrund starten und später beenden.
cargo run --locked --features web --bin airust -- -d
cargo run --locked --features web --bin airust -- stop

# Mit der Produktübersicht starten.
cargo run --locked --features web --bin airust -- --landingpage
```

Erlaubt sind Ports von 1 bis 65534. `--help` zeigt weitere Befehle. Die Konsole kennt `help`, `clear`, `status`, `restart` und `reset`. `reset` löscht die Dashboard-Datenbank. Freie Shell-Befehle werden nicht ausgeführt.

Das Dashboard verwendet `airust.db` im aktuellen Arbeitsordner. Die bestehende Datei wurde bei dieser Überarbeitung nicht geändert. Tests verwenden eigene temporäre Datenbanken.

Standardmäßig sind Webserver und Konsole nur lokal erreichbar. `AIRUST_BIND_ADDR` legt die Bindeadresse fest. Beispiel für eine ausdrücklich gewünschte Netzwerkfreigabe:

```bash
AIRUST_BIND_ADDR=0.0.0.0 cargo run --locked --features web --bin airust
```

Das Dashboard hat keine Benutzeranmeldung und kann Dateien und Datenbanken verwalten. Für eine öffentliche Bereitstellung benötigt es einen geschützten Zugang außerhalb von AIRust. Beliebige fremde Browser-Seiten erhalten keine pauschale CORS-Freigabe mehr.

## 6. Docker verwenden

```bash
docker build -t airust:0.1.8 .
docker run --rm \
  -p 127.0.0.1:7070:7070 \
  -p 127.0.0.1:7071:7071 \
  airust:0.1.8
```

Der Container baut ausdrücklich die Web-Erweiterung. Die Ports bleiben durch diese Zuordnung auf dem Host lokal. Das Image enthält weder deine vorhandene Datenbank noch das Sicherungsarchiv. Die Anleitung wird mitkopiert, weil der Server sie für seinen Guide benötigt.

Ohne eigenes Datenvolume lebt eine neu angelegte Datenbank nur im Container. Der Docker-Build benötigt Zugang zur Paketregistrierung.

## 7. Qualität prüfen

```bash
./test.sh
cargo run --locked --release --example benchmark -- 1000 100
```

Der Laufzeitvergleich misst einen kleinen festgelegten Fall. Seine Zeiten hängen vom Rechner ab. Die durchgeführten lokalen Prüfungen stehen im [Änderungsbericht](docs/AENDERUNGEN-0.1.8.md).
