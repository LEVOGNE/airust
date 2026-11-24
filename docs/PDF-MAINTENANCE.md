# PDF backend maintenance

AIRust pins `airust-pdf-backend = 0.3.5`, an independently named derivative of `pdf_oxide = 0.3.78`. The source base is upstream commit `ad49c4cb3638dc950a29350ef882161372fde473`. The backend retains the original licenses and notices, its complete source-hash manifest and the exact maintained patch. This work program changes no backend runtime source or backend version.

## Patch inventory

| Maintained area | Reason | Owned regression coverage | Upstream replacement gate |
| --- | --- | --- | --- |
| `src/airust_small_object.rs`, adapter declaration and `src/document.rs` loading | Bound complete small-object lookahead to 4 KiB on long physical lines while retaining the existing parser and fallback. | Complete object types, streams, incomplete/end-marker cases, beyond-budget objects, deterministic mutations, tagged long lines and parallel cold file readers. | Equivalent complete objects and structure order, every fallback retained, same byte-boundary behavior and paired real-document cost measurements. |
| `src/document.rs` object-header boundary | Avoid slicing replacement UTF-8 text at an invalid boundary. | `malformed_utf8_header_no_longer_unwinds`, `utf8_literal_data_does_not_change_boundaries`. | Malformed UTF-8 input stays contained without changing supported complete extraction. |
| `src/lib.rs` C ABI gate | Default Rust use must coexist with the upstream parser without duplicate C symbols. | Explicit `c-api` build and earlier differential linking controls. | C exports remain explicitly selected and both Rust libraries link together. |
| `src/extractors/text.rs` Form failures and cache handling | Propagate existing depth/processing-budget errors, restore local state and avoid caching a partial successful prefix. | Supported nesting, excessive depth, processing budget and repeated extraction. | Complete supported text and explicit errors at the same limits across extraction APIs; failed caches never become successful prefixes. |
| `src/document.rs` fallback page-count exhaustion | Preserve exhaustion as an error instead of accepting a partial count. | `fallback_page_count_rejects_depth_exhaustion`. | Exhaustion remains explicit; valid fallback page counts remain complete. |

The current owned suite has 14 regression tests. It is separate from unavailable upstream fixture-dependent tests. The provenance check verifies 381 original source-file identities, permits the three recorded modified files and the owned helper, compares `upstream.patch`, and preserves MIT/Apache license texts, NOTICE and trademark material. [Exact provenance](https://github.com/LEVOGNE/airust/blob/v0.5.2/crates/airust-pdf-backend/UPSTREAM.md).

## Upstream status

On 24 November 2026, both [the latest GitHub release](https://github.com/yfedoseev/pdf_oxide/releases/tag/v0.3.78) and [the crates.io registry](https://crates.io/crates/pdf_oxide) identify 0.3.78 as the current published upstream version. That is the maintained fork's existing base. This check does not establish that unreleased upstream development contains equivalent fixes. No upstream issue, message or pull request was sent by this work program.

The next candidate upstream version is adopted only after passing the replacement gates below. A feature list or newer version number is not evidence of equivalent extraction, limits or linking behavior.

## Reproduce maintenance checks

Use a dedicated temporary build directory and preserve the command results:

```bash
export CARGO_TARGET_DIR=/tmp/airust-pdf-backend-build
cargo test --manifest-path crates/airust-pdf-backend/Cargo.toml --test regressions --locked
cargo check --manifest-path crates/airust-pdf-backend/Cargo.toml --features c-api --lib --locked
python3 -B scripts/check_pdf_backend.py
```

The backend CI also checks Rust 1.89 and packages its exact source. Main AIRust checks separately replay the bounded PDF seed corpus and validate importer contracts. Keep the two lockfiles and package identities distinct. Use `cargo fmt --package airust` for owned application code; the provenance helper checks formatting of the owned backend helper and regressions without rewriting preserved third-party source.

## Retire patches when equivalence is established

1. Record the candidate upstream archive, source commit, dependency lock and compiler. Build an adapter against the unchanged AIRust importer contract in isolation.
2. Run every owned regression and retained malformed/large-file control, including repeated calls after an error. Verify source text, structure order, page counts, original knowledge and error categories.
3. Repeat paired complete-import and bounded-preview measurements on the retained real-document corpus. Keep every slower outcome and separate parser opening, extraction, conversion and process resource observations.
4. Verify default Rust linking and opt-in C ABI behavior beside the original library. Repeat minimum-Rust, platform and package checks.
5. Remove one redundant patch at a time, preserving its regression test. Retire the separate backend dependency only after all importer, compatibility and evidence gates pass. Record any intentional text-fidelity changes and required rebuilds explicitly.

The importer's budgets and child supervision keep their documented cooperative/native boundaries. This maintenance contract does not claim complete extraction of arbitrary damaged PDFs or hard OS memory quotas. Original PDF knowledge and historical measurements remain retained.
