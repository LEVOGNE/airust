#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

cargo fmt --package airust -- --check
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

cargo check --lib --no-default-features --features retrieval --locked
cargo check --lib --no-default-features --features retrieval,advanced-search --locked
cargo check --lib --no-default-features --features retrieval,embeddings --locked
cargo check --lib --no-default-features --features retrieval,hybrid --locked
cargo check --lib --no-default-features --features retrieval,reranking --locked
cargo test --no-default-features --features retrieval --test retrieval --locked
cargo run --locked --features retrieval --example retrieval

cargo test --no-default-features --features retrieval --test excerpts --locked
cargo run --locked --features retrieval --example relevant_excerpts
cargo run --locked --features retrieval,advanced-search --example relevant_excerpts

cargo check --lib --no-default-features --features snapshots --locked
cargo check --lib --no-default-features --features snapshots,retrieval --locked
cargo check --lib --no-default-features --features snapshots,advanced-search --locked
cargo test --no-default-features --features snapshots --test snapshots --locked
cargo run --locked --features snapshots,retrieval --example snapshots
cargo run --locked --features snapshots,retrieval,advanced-search,context-assembly --example snapshots

cargo test --no-default-features --features retrieval --test assessment --locked
cargo run --locked --features retrieval --example evidence_assessment
cargo run --locked --features retrieval --example assessment_quality > /dev/null

cargo check --no-default-features --features execution --locked
cargo clippy --all-targets --no-default-features --features execution --locked -- -D warnings
cargo test --no-default-features --features execution --test execution --locked
cargo run --locked --features execution,retrieval --example cancellation

cargo test --no-default-features --features execution --test time_budgets --locked
cargo run --locked --features execution,retrieval --example time_budget > /dev/null

cargo test --no-default-features --features execution --test progress --locked
cargo run --locked --features execution,retrieval --example progress > /dev/null

cargo run --locked --example stress_index -- scale 1000 > /dev/null
cargo run --locked --example stress_index -- clone 1000 > /dev/null
cargo run --locked --features snapshots,execution --example stress_snapshots -- 1000 > /dev/null

cargo check --no-default-features --features shared-index --locked
cargo clippy --all-targets --no-default-features --features shared-index --locked -- -D warnings
cargo test --no-default-features --features shared-index --test shared_index --locked

cargo test --no-default-features --features snapshots,advanced-search --test incremental --locked
cargo run --locked --features snapshots,advanced-search --example incremental
cargo run --locked --features snapshots,advanced-search,shared-index --example incremental

cargo check --no-default-features --features adaptive --locked
cargo clippy --all-targets --no-default-features --features adaptive --locked -- -D warnings
cargo test --no-default-features --features adaptive --test adaptive --locked
cargo run --locked --features adaptive --example adaptive
cargo run --locked --features adaptive,advanced-search --example adaptive_quality > /dev/null
cargo run --locked --features adaptive,hybrid --example adaptive_benchmark > /dev/null

cargo clippy --all-targets --no-default-features --features feedback --locked -- -D warnings
cargo test --no-default-features --features feedback --test feedback --locked
cargo run --locked --features feedback --example feedback > /dev/null
cargo run --locked --features feedback --example feedback_benchmark > /dev/null

cargo clippy --all-targets --no-default-features --features learned-ranking --locked -- -D warnings
cargo test --no-default-features --features learned-ranking --test learned_ranking --locked
cargo run --locked --features learned-ranking --example learned_ranking > /dev/null

cargo clippy --all-targets --no-default-features --features reasoning --locked -- -D warnings
cargo test --no-default-features --features reasoning --test reasoning --locked
cargo run --locked --features reasoning --example reasoning > /dev/null

cargo clippy --all-targets --no-default-features --features quality-gates --locked -- -D warnings
cargo test --no-default-features --features quality-gates --test quality_gates --locked
cargo run --locked --features quality-gates --example quality_gate > /dev/null

cargo clippy --all-targets --no-default-features --features evidence-freshness --locked -- -D warnings
cargo test --no-default-features --features evidence-freshness --test evidence_freshness --locked
cargo run --locked --features evidence-freshness --example evidence_freshness > /dev/null

cargo clippy --all-targets --no-default-features --features rank-explanations --locked -- -D warnings
cargo test --no-default-features --features rank-explanations --test rank_explanations --locked
cargo run --locked --features rank-explanations --example rank_explanations > /dev/null

cargo clippy --all-targets --no-default-features --features training-pipeline --locked -- -D warnings
cargo test --no-default-features --features training-pipeline --test training_pipeline --locked
cargo run --locked --features training-pipeline --example training_pipeline > /dev/null

cargo clippy --all-targets --no-default-features --features disk-index --locked -- -D warnings
cargo test --no-default-features --features disk-index --test disk_index --locked
cargo run --locked --features disk-index --example disk_index > /dev/null
cargo run --locked --features disk-index --example disk_benchmark -- 1000 disk > /dev/null

cargo test --no-default-features --features documents --test documents --locked
cargo run --locked --features documents --example documents

cargo test --no-default-features --features mcp --test mcp --test mcp_cli --test controlled_matching --locked
cargo run --locked --features mcp --example mcp

python3 -B scripts/check_mcp_dependencies.py

python3 -B scripts/check_features.py

python3 -B scripts/check_package.py

cargo test --no-default-features --features retrieval --test citations --test evidence_selection --locked
cargo run --locked --features retrieval --example checked_evidence > /dev/null

cargo test --no-default-features --features retrieval --test lexical_query --locked

cargo test --no-default-features --features retrieval --test lexical_coverage --locked
