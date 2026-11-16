# An AIRust mitarbeiten

Verwende Rust 1.89 oder neuer. Prüfe eine Änderung im Projektordner mit:

```bash
./test.sh
```

Der Standard-Build muss eine Bibliothek ohne PDF, HTTP, Datenbank oder LLM bleiben. Neue Suchfunktionen sollen den gemeinsamen Index und die gemeinsame Textzerlegung verwenden.

Ein Rangwert ist keine Wahrscheinlichkeit. Eine Wortüberschneidung ist kein Beweis für eine fachliche Beziehung. Neue Funktionen sollen ihre Grenzen und ihre Ergebnisse klar benennen.

Teste bei Änderungen am Index besonders Einfügen, Ersetzen und Löschen. Prüfe bei Textverarbeitung Unicode und Wortgrenzen. Bestehende Trainingsdateien und Metadaten sollen lesbar bleiben.

Passe Beispiele, README, Startanleitung und Änderungsbericht an, wenn sich das öffentliche Verhalten ändert. Neue Integrationen in DenkMaps oder MCP gehören in deren eigene Aufruf- und Transportschicht.

Für eine Veröffentlichung müssen die Prüfungen und `cargo package --locked --allow-dirty` erfolgreich sein. Kontrolliere den Paketinhalt. Lokale Datenbanken und Sicherungen dürfen nicht darin liegen. `cargo publish` ist ein eigener, ausdrücklich auszuführender Schritt.
