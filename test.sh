#!/bin/bash

echo "===== SimpleAgent Test ====="
cargo run --bin cli -- simple "Was ist airust?"
cargo run --bin cli -- simple "Was ist Airust?"  # Groß-/Kleinschreibung

echo -e "\n===== FuzzyAgent Test ====="
cargo run --bin cli -- fuzzy "Was ist airust?"
cargo run --bin cli -- fuzzy "Was ist Ayrast?"  # Schreibfehler

echo -e "\n===== TfidfAgent Test ====="
cargo run --bin cli -- tfidf "Was ist airust?"
cargo run --bin cli -- tfidf "Erkläre mir airust"  # Andere Formulierung
cargo run --bin cli -- tfidf "Was ist TF-IDF?"
cargo run --bin cli -- tfidf "TF-IDF erklären"  # Keywords

echo -e "\n===== ContextAgent Test ====="
# Hinweis: Der ContextAgent funktioniert am besten mit einer manuellen Test-Anwendung
