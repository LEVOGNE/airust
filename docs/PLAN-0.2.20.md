# AIRust 0.2.20 acceptance plan

Reduce retained-copy RAM using an explicit optional shared-index flag. Keep default storage and all public signatures/wire formats unchanged. Share immutable analyzed record, posting and duplicate buffers; detach writes and preserve old/sibling views. Outer maps remain copied, and common-term mutations may still be costly.

Verify private pointer sharing/detachment, returned removed records, metadata, duplicates, exact scores, many retained generations, independent concurrent writers, cache and semantic/snapshot composition. Reuse identical 0.2.19 stress generators in default/flagged builds; publish binary/source hashes, CPU/RSS and single-owner regressions. Repeat complete/isolated/minimum-Rust/compatibility/audit/consumer/package/CI/registry gates. No application integration.
