# Token-overlap matching in AIRust 0.3.11

Token-overlap matching compares the words shared by two texts. Its ranking score is the size of the shared-word set divided by the size of their combined word set. Character similarity is an additional explanation; it does not decide this strategy's order.

Earlier versions calculated character distance for every qualifying candidate. Version 0.3.11 selects the best token matches first, then calculates character similarity for those returned entries. This avoids unnecessary work for discarded candidates. Returned IDs, scores, exact-match flags, character/token similarities and stable ID ordering retain their meanings and exact values. Fuzzy matching still ranks by character similarity. No public API, flag or JSON field changes.

## Use the existing API

```toml
[dependencies]
airust = { version = "=0.3.11", default-features = false }
```

```rust
use airust::{Engine, KnowledgeEntry, MatchOptions, MatchStrategy};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::from_entries([
        KnowledgeEntry::new("a", "memory allocation"),
        KnowledgeEntry::new("b", "allocation memory memory"),
        KnowledgeEntry::new("c", "memory allocation function"),
    ])?;
    let matches = engine.match_with_options("memory allocation", &MatchOptions {
        strategy: MatchStrategy::TokenOverlap,
        min_similarity: 0.5,
        limit: 10,
    })?;
    assert_eq!(matches[0].entry.id, "a");
    assert!(matches[0].similarity.exact);
    assert_eq!(matches[1].entry.id, "b");
    assert_eq!(matches[1].score, 1.0);
    assert!(!matches[1].similarity.exact);
    assert!(matches[1].similarity.character < 1.0);
    Ok(())
}
```

## Choose a strategy and budget

Use `Exact` for normalized identical text, `TokenOverlap` for shared words, and `Fuzzy` for character edits. These are lexical operations. Shared words do not prove semantic equivalence; optional embedding APIs serve that separate purpose.

`limit` bounds returned matches, not candidate collection or total CPU/RAM. A larger token-overlap result limit also requests more character explanations. Thresholds must be finite and between zero and one. Zero limit returns no matches. Direct matching remains synchronous and has no hard deadline or OS memory sandbox. The existing controlled search APIs do not turn all matching calls into interruptible work.

## Validation and retained knowledge

Three regressions compare results against complete text comparisons over Unicode, thresholds, limits, ties, updates and JSON round trips. Frozen previous/current binaries compare all returned IDs, ranks and floating-point similarity bits on the retained real corpus. Method timing excludes output serialization; end-to-end index load/build costs are reported separately.

The real-knowledge validation tool also checks updates against a complete rebuild, JSON reconstruction, concurrent readers, cancellation/deadline handling for controlled searches and isolated clones. These are workload-specific correctness and cost checks, not semantic quality labels or a capacity/security guarantee. Original knowledge remains read-only; temporary changes never enter canonical exports.

See [executed results](CHANGES-0.3.11.md), [reproduction](validation-0.3.11/README.md) and [0.3.x readiness](STABILITY-0.3.md). No DenkMaps/MCP integration is added.
