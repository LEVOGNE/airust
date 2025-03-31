#!/bin/bash

echo "===== SimpleAgent Test ====="
cargo run --bin airust -- query simple "What is airust?"
cargo run --bin airust -- query simple "What is Airust?"  # Case insensitive

echo -e "\n===== FuzzyAgent Test ====="
cargo run --bin airust -- query fuzzy "What is airust?"
cargo run --bin airust -- query fuzzy "What is Ayrast?"  # Spelling error

echo -e "\n===== TfidfAgent Test ====="
cargo run --bin airust -- query tfidf "What is airust?"
cargo run --bin airust -- query tfidf "Explain airust to me"  # Different wording
cargo run --bin airust -- query tfidf "What is TF-IDF?"
cargo run --bin airust -- query tfidf "TF-IDF explain"  # Keywords

echo -e "\n===== ContextAgent Test ====="
cargo run --bin airust -- query context "What is airust?"  # Context agent without context
echo "Note: For better context tests, use 'cargo run --bin airust -- interactive' and select option 4"