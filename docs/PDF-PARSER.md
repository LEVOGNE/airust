# PDF parser optimization in AIRust 0.3.4

AIRust 0.3.4 removes a measured bottleneck in the optional PDF importer. Existing `PdfLoader`, `PdfImportLimits`, saved training files and CLI flags keep their API. The default knowledge engine is unchanged. No DenkMaps or MCP integration is added.

## Use it

```toml
[dependencies]
airust = { version = "=0.3.5", default-features = false, features = ["pdf"] }
```

```rust
use airust::{PdfImportLimits, PdfLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = PdfLoader::new().with_limits(PdfImportLimits::default());
    let knowledge = loader.pdf_to_knowledge_base("document.pdf")?;
    loader.save_knowledge_base_if_absent(&knowledge, "knowledge.json")?;
    Ok(())
}
```

Call `loader.pdf_to_knowledge_base(path)?`, then save the complete result with `save_knowledge_base_if_absent` or explicit replacement. Use [supervised CLI jobs](PDF-WORKERS.md) when a blocking parser call needs a deadline. Existing byte/page/text/chunk limits remain [opt-in](PDF-LIMITS.md).

## What changed

The slow input contains many small uncompressed objects on very long physical lines. The previous loader repeatedly read and split those lines while building the document structure tree. A five-second profile located most sampled work in object loading/string splitting under structure traversal.

The maintained backend first peeks at the buffered input. Ordinary short lines keep the existing path without another file read. On a long physical line, it examines at most 4 KiB. The existing parser must recognize a complete nonstream object followed by a separate `endobj` token. Streams, larger/incomplete probes, ambiguous markers and unusual headers use the original loader. The accepted object then follows the same decryption and cache path.

Source PDF bytes are never rewritten. The complete structure tree, reading order, replacement text and page handling remain enabled. This is an object-loading optimization, not a fallback to simpler reading order.

A separate fix replaces a UTF-8 string slice with a byte comparison when checking the preceding `end` marker. The known malformed-object-header cases now return normally rather than unwinding. The library panic boundary remains for unexpected future Rust parser panics, and supervised jobs still contain worker failure.

## Measured results

Five fresh-process samples per version and case use the same thin-LTO/codegen1 release profiles on Rust 1.98/macOS ARM64. Order alternates between versions. Timings include conversion, chunking, JSON and atomic output. Input/file caches are reused; no completely idle-machine or cold-disk claim is made.

| Input | Reader | 0.3.3 median | 0.3.4 median | Interpretation |
| --- | --- | --- | --- | --- |
| Known slow 494-page manual | Bounded memory snapshot | 23.362 s | 0.582 s | About 40 times faster for this case. |
| 68-page manual | File | 0.501 s | 0.088 s | About 5.7 times faster for this case. |
| 913-page manual | File | 1.206 s | 1.185 s | Similar; no broad speed claim. |
| 2,488-page Python reference | Bounded memory snapshot | 7.091 s | 6.983 s | Similar; no broad speed claim. |

All 40 paired output files match their counterpart byte for byte. A separate direct-parser comparison covers all 15 inputs and 8,960 physical pages: every raw page-text digest, including whitespace, is identical. The SDK pass also preserves all 33,708 chunks, provenance, page sets, checked ranked IDs and page ranks. The existing 115 literal page probes remain BM25 111/115 and TF-IDF 102/115 in the expected top-ten pages. These are page-retrieval probes, not independent semantic relevance judgments.

## Robustness and scope

Ten maintained-backend tests cover complete/truncated/large objects, stream fallback, literal end markers, Unicode, complete tagged trees, differential values, 2,000 deterministic probe mutations and parallel cold file-backed loads. Linux linking exposed colliding C symbols when both parsers were present. The new backend makes its C ABI explicitly opt-in through `c-api`; default Rust use coexists with the original library. That path is tested in Linux CI and a Rust 1.89 registry consumer.

The 4,096-case malformed-input campaign reports no parser panic, unexpected parent exit or outer watchdog timeout. All failed conversions preserve previous output. Seventeen former caught header panics are eliminated. Two formerly rejected mutations now recover two nonempty pages; an independent parser also finds those pages. Accepted malformed files are not certified complete or semantically correct.

The final main-checkout CLI also passes the retained 134-case malformed-input set. Killing its owned worker returns a normal failure, preserves previous output and leaves no worker process. The current library reopens the retained 33,708-entry canonical index and returns ten bounded query hits. Original PDF hashes remain unchanged.

A real one-second worker deadline still stops the 2,488-page reference and preserves the prior output. The previously slow 494-page case can now complete within that budget. Library limits and process supervision are not hard whole-command real-time, RAM, disk or operating-system sandbox guarantees. Large/ambiguous objects retain the original fallback and may remain expensive.

## Backend provenance and maintenance

`airust-pdf-backend = 0.3.4` is an independently named optional derivative of published `pdf_oxide` 0.3.78, commit `ad49c4cb3638dc950a29350ef882161372fde473`. Only two of its 381 original source files are modified, with one added helper. Original licenses, notices and trademark terms remain intact; the maintained package is not endorsed by the upstream project.

The backend source lives in `crates/airust-pdf-backend`; the main AIRust crate resolves the independently published backend only when PDF processing is selected. The default core dependency set does not gain the parser. The backend's upstream source hashes and patch are checked by `scripts/check_pdf_backend.py`. Its full external-fixture-dependent upstream suite is not claimed as executed; the supported Rust configuration has separate regression and real-document gates. Audit both the main lockfile and the backend lockfile, which retains the original package as a comparison dependency for advisory tracking.

The fresh main-lockfile audit reports zero known vulnerabilities and three maintenance warnings for optional/transitive packages. These warnings remain visible in the [audit record](measurements/0.3.4/audit.json); an advisory scan is not an independent security audit.

Format owned code with `cargo fmt --package airust`. The provenance checker formats/checks only the maintained helper and tests; formatting the entire copied upstream tree would break its exact provenance record. Never replace canonical knowledge with edited soak/fuzz output. All source inputs, canonical indexes, old binaries and raw evidence remain in the ignored local lab. [Aggregate measurements](measurements/0.3.4/) identify versions, costs and limits.


AIRust 0.3.5 uses this same independently versioned 0.3.4 backend. It improves the surrounding page/chunk budget pipeline; see [current import results and API](PDF-IMPORT-PIPELINE.md). The 0.3.4 performance results above remain historical comparisons.
