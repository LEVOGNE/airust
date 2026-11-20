# Reproduce 0.3.2 checks

Run `./test.sh` on Rust 1.92+ and Python 3.11+. The suite includes file/page/text/chunk boundaries, overlap amplification, UTF-8 byte counts, non-Unicode CLI arguments, escaped terminal output, malformed headers and output-preservation races. Every declared feature is also checked independently.

The [PDF limit contract](../PDF-LIMITS.md) explains the policy and its native-parser limits. The [retained-knowledge guide](../PDF-STABILITY.md) describes the private lab and reusable measurement tools. No private source PDFs, extracted text, queries or indexes are redistributed.

`consumer.rs` and `consumer.toml` form an independent exact-version Rust 1.89 application, including the new PDF policy API. Copy them into a temporary application's src/main.rs and Cargo.toml, then run it against the registry version. For pre-publication checks, add the checkout's absolute path to the dependency. Other minimum-Rust, patch API, audit, packaging and exact-commit CI gates follow the [release workflow](../validation-0.3.0/README.md).

Public aggregate [measurements](../measurements/0.3.2/) identify corpus preservation and malformed-input outcomes. No speed improvement is claimed from a single warm corpus pass or these policy limits. Docker context exclusion was reviewed statically; Docker was not installed on the host.
