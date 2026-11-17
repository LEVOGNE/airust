#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

cargo fmt --all -- --check
cargo test --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --all-targets --features cli --locked -- -D warnings
cargo check --lib --no-default-features --features pdf --locked
cargo check --lib --no-default-features --features bots --locked
cargo check --lib --no-default-features --features async --locked
cargo check --lib --no-default-features --features embeddings --locked
cargo check --lib --no-default-features --features hybrid --locked
cargo check --lib --no-default-features --features graph --locked
cargo check --lib --no-default-features --features context-assembly --locked
cargo check --lib --no-default-features --features provenance --locked
cargo check --lib --no-default-features --features cache --locked
cargo check --lib --no-default-features --features hnsw --locked
cargo check --lib --no-default-features --features versioning --locked
cargo check --lib --no-default-features --features quality --locked
cargo check --lib --no-default-features --features advanced-search --locked
cargo check --lib --no-default-features --features reranking --locked
cargo check --lib --no-default-features --features local-reranking --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --locked
cargo run --locked --example core
cargo run --locked --features embeddings --example semantic
cargo run --locked --features hybrid --example hybrid
cargo run --locked --features graph --example graph
cargo run --locked --features context-assembly --example context
cargo run --locked --features provenance --example provenance
cargo run --locked --features provenance,context-assembly --example provenance_context
cargo run --locked --features cache --example cache
cargo run --locked --features hnsw --example ann
cargo run --locked --features versioning --example versioning
cargo run --locked --features quality --example quality
cargo run --locked --features quality,hybrid --example retrieval_quality > /dev/null
cargo run --locked --features hnsw,cache,context-assembly,provenance,quality --example knowledge_pipeline
cargo run --locked --features quality,context-assembly,provenance --example quality_benchmark -- 1000 3 > /dev/null
cargo run --locked --features reranking --example reranking
cargo run --locked --features hnsw,cache,context-assembly,provenance,quality,reranking --example knowledge_pipeline
python3 -B tests/download_reranker_test.py
cargo run --locked --features advanced-search --example advanced_search

cargo check --lib --no-default-features --features storage --locked
cargo check --lib --no-default-features --features sqlite-storage --locked
cargo run --locked --features storage --example storage
