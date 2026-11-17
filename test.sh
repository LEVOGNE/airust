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
