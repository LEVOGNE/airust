# Published-baseline index controls

The private cursor and IDF-memo extraction preserves the four complete consumer signatures against registry AIRust 0.5.2. Default and shared-index builds produce the same ordered IDs, score bits, matched terms and portable records through 3,792 query/stage/scope/algorithm combinations per build. The consumer also checks invalid-update rollback. [Exact signatures and every slower query mean](summary.json).

The two independent alternating campaigns also pass the frozen 10% aggregate p95/RSS regression gate. These measurements are behavior-preserving maintenance evidence; they do not establish a speed improvement.

| Campaign | Storage | Baseline p95 µs | Candidate p95 µs | p95 ratio | Baseline median peak MiB | Candidate median peak MiB | RSS ratio |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | default | 4207.084 | 4217.042 | 1.00237 | 61.33 | 61.09 | 0.99618 |
| 1 | shared | 4172.208 | 4154.917 | 0.99586 | 60.36 | 60.33 | 0.99948 |
| 2 | default | 4222.583 | 4230.833 | 1.00195 | 61.31 | 61.03 | 0.99541 |
| 2 | shared | 4211.542 | 4231.791 | 1.00481 | 60.27 | 60.38 | 1.00181 |

Each of 56 measured workers uses 9,304 entries, warms 158 queries for both algorithms and records seven repetitions. The p95 is pooled over individual warm query samples; process construction and final report encoding are excluded. Median and maximum exact-child peak RSS are retained. All slower per-query means, including noisy small-query ratios, remain in the summary. No model inference or compiler ran concurrently with timed workers.

The compiler is Rust 1.98.0 on the same local macOS ARM64 host as the reference campaign. The sandbox did not expose every optional host observation to this runner; unavailable fields are explicit rather than inferred. Raw signatures, all 56 worker reports and logs remain under ignored `local-lab/reference-workflow/index-refactor-01`, with a public [artifact manifest](artifact-manifest.json). Exact compiler/source/lock/binary identities are retained in [provenance.json](provenance.json).

See [the module boundaries and reproduction commands](../../INDEX-INTERNALS.md). These finite controls do not prove equivalence for arbitrary inputs or behavior under every scheduling condition. Existing independent-oracle, cancellation, mutation and concurrent-reader tests are still required.
