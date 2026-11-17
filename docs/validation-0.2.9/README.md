# Independent 0.2.9 consumer

The consumer uses the exact published registry version with default features disabled. It exercises existing lexical, semantic, hybrid, graph, context, provenance, cache, revision and quality APIs plus optional reranking. It uses handwritten vectors/scores to check API contracts, not model quality. It requires no model or runtime.

After 0.2.9 is published, create a fresh temporary consumer outside the checkout:

```bash
mkdir -p /tmp/airust-029-consumer/src
cp docs/validation-0.2.9/consumer.toml /tmp/airust-029-consumer/Cargo.toml
cp docs/validation-0.2.9/consumer.rs /tmp/airust-029-consumer/src/main.rs
cargo +1.89.0 run --manifest-path /tmp/airust-029-consumer/Cargo.toml
```

Choose a different temporary path if that directory already exists. The flat fixture filenames let Cargo include the consumer source in the published package rather than excluding a nested package. Cargo creates that consumer's own lockfile; no local path override should be present. Confirm `cargo metadata` reports the library's registry source. The release process compares its packaged VCS commit and registry checksum with the GitHub tag and crate asset.

During development the same source was tested with a temporary local dependency before registry publication. Native model evaluation is separately recorded in [the release report](../CHANGES-0.2.9.md).
