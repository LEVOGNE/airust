# AIRust 0.2.16 acceptance plan

Add optional one-way cancellation tokens and shared execution controls. Integrate cooperative checks into controlled lexical/phrase/semantic/hybrid search, reranking, compact retrieval and off-lock snapshot staging. Preserve default APIs and do not disguise cancellation as provider fallback or insufficient evidence. Check immediately before atomic publication, never report cancellation after an already completed commit.

Test concurrent first cancellation, cancelled-before-call including zero limits, cancellation inside candidate collection, provider/batch cancellation, simultaneous request/scorer controls, old-reader preservation and failed publication. Compare ranking bits to unchanged APIs and measure execution overhead. Run full and isolated tests, minimum Rust checks, strict lint/docs, patch compatibility, audit, consumer/package/CI/registry verification. No application integration or hard preemption promise.
