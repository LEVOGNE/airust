#!/bin/bash

echo "===== SimpleAgent Test ====="
cargo run --bin cli -- simple "What is airust?"
cargo run --bin cli -- simple "What is Airust?"  # Case insensitive

echo -e "\n===== FuzzyAgent Test ====="
cargo run --bin cli -- fuzzy "What is airust?"
cargo run --bin cli -- fuzzy "What is Ayrast?"  # Spelling error

echo -e "\n===== TfidfAgent Test ====="
cargo run --bin cli -- tfidf "What is airust?"
cargo run --bin cli -- tfidf "Explain airust to me"  # Different wording
cargo run --bin cli -- tfidf "What is TF-IDF?"
cargo run --bin cli -- tfidf "TF-IDF explain"  # Keywords

echo -e "\n===== ContextAgent Test ====="
# Note: The ContextAgent works best with a manual test application