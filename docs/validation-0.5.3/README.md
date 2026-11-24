# Release verification for AIRust 0.5.3

The release source is the commit identified by annotated tag `v0.5.3` and the crate's `.cargo_vcs_info.json`. `master` is advanced to that same commit. The exact maintained PDF backend stays at 0.3.5. Development evidence remains in [the pre-release closeout](../REFERENCE-VALIDATION.md); the original 0.5.2 [receipt](../validation-0.5.2/closeout.json) and frozen reference dataset are unchanged.

The completed release manifest is distributed as a [GitHub release asset](https://github.com/LEVOGNE/airust/releases/download/v0.5.3/release-manifest.json), alongside `SHA256SUMS` and the exact published `.crate`. It records source/tag identities, local and hosted gate results, advisory results, registry metadata, independent consumer checks and artifact hashes. This guide defines the checks; the completed manifest supplies their observed outcomes without introducing a self-referential package checksum.

## Before publication

From a clean release commit, with a dedicated Cargo target directory:

```bash
./test.sh
cargo package --locked
cargo publish --locked --dry-run
```

Inspect all archive members. Local databases, credentials, model weights, private knowledge, backups, logs and build output stay outside the crate. The unchanged public sample `knowledge/train.json` and PDF fixture remain intentional package inputs. Verify `.cargo_vcs_info.json` identifies the clean release commit and that package-normalized dependencies select registry `airust-pdf-backend = 0.3.5`.

Hosted checks run on that exact commit for Linux, macOS, Rust 1.89 provider-neutral targets, Rust 1.92 all-feature targets and the backend's own tests/package/minimum compiler. Separate local backend checks run its 14 owned regressions, explicit C ABI and the 381-file upstream provenance check. Audit the root and backend lockfiles with the recorded RustSec tool/database identity. Build-only optional adapters do not download or execute model assets.

## Independent registry consumer

The retained [consumer source](consumer.rs) and [manifest](consumer.toml) use exactly `airust = 0.5.3` from crates.io. Copy them to a new directory as `src/main.rs` and `Cargo.toml`, then run with Rust 1.89. Keep the resolved consumer lockfile outside the package and attach it to the release evidence:

```bash
cargo +1.89.0 run --manifest-path /absolute/path/to/consumer/Cargo.toml
```

It checks the actual exported version, lexical search and Unicode normalization, retained clones, failed-update rollback, generic JSON restoration, exact document subspans and restoration, typed cancellation and all five MCP schema identifiers. Additional current README/cookbook consumer checks use the same exact registry dependency. A successful consumer does not establish correctness for every input or every feature subset.

## After publication

1. Download the crates.io `.crate` independently, compare its bytes and registry checksum to the prepared archive, and inspect its source commit/version.
2. Verify the GitHub `.crate` asset has the same SHA-256 and bytes. Verify tag `v0.5.3` resolves to the release commit and that both branch/tag CI runs conclude successfully.
3. Run the registry consumer and retain its lock, command output and compiler identity. Check package-normalized optional dependencies against the published registry graph.
4. Attach the completed release manifest and checksum list, and verify the public release metadata and assets again. Preserve logs, packages, manifests and original knowledge before removing dedicated build output.

The reference model campaign, paired refactor measurements and historical fuzz/Miri campaigns are not repeated by a version change. Their original source/model identities and resource limits remain explicit. No changed backend or new default ranking profile is published with 0.5.3.
