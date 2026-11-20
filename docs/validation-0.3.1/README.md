# Reproduce 0.3.1 checks

Run `./test.sh` on Rust 1.92+ and Python 3.11+. The complete suite includes authored blank, damaged, encrypted and malformed-header PDFs, exclusive publication tests and a generated 500-page CLI race regression. It also checks every declared feature independently.

The optional manual tools are `pdf_stability`, `pdf_parser_profile` and `saved_knowledge`; see [commands and API limits](../PDF-STABILITY.md). The actual local lab is in the ignored `local-lab/pdf-stability` directory. Private source PDFs and extracted indexes are not part of the repository or crate.

`consumer.rs` and `consumer.toml` form an independent Rust 1.89 application. Copy them into a temporary application's src/main.rs and Cargo.toml, then run it against the exact registry version. For pre-publication checks, add the checkout's absolute path to the dependency. Other minimum-Rust, API-compatibility, audit, package and exact-commit CI gates follow the [0.3.0 release workflow](../validation-0.3.0/README.md).
