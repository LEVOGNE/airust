# Reference application and evaluation

AIRust's reference application is local search over technical project documentation. A Rust application imports UTF-8 Markdown, keeps exact text revisions, retrieves a small set of passages, and exposes the same collection through an optional read-only MCP server. A useful result identifies the source document, revision and original UTF-8 byte range so the application can show and verify its evidence.

The intended user is a developer integrating document search into a desktop application or local agent. The first supported collection contains English technical documentation and German and English questions. The application owns collection selection, file access and durable storage. A search result is evidence for review; this workflow does not generate answers or certify that a quotation answers a question.

## User tasks

1. Import a directory of Markdown files into a new workspace without replacing an existing workspace.
2. Ask a technical question and receive at most six source passages, each at most 2,000 UTF-8 bytes and at most 12,000 text bytes in total.
3. Resolve every returned passage against its exact document revision. Edited or missing workspace files cause a clear error instead of stale citations.
4. Connect an MCP client to the same saved collection using local standard input and output.
5. Rebuild a separate workspace from changed documents while retaining the previous workspace for existing consumers.

## Run the complete example

From this source checkout:

```bash
export AIRUST_DEMO_DIR="$(mktemp -d /tmp/airust-reference.XXXXXX)"
cargo run --locked --features mcp --example reference_workspace -- build tests/fixtures/reference-v1/sources "$AIRUST_DEMO_DIR/workspace" manuals
cargo run --locked --features mcp --example reference_workspace -- query "$AIRUST_DEMO_DIR/workspace" "Welche Bytepositionen verwendet TextSpan?"
cargo run --locked --features mcp --example reference_workspace -- serve "$AIRUST_DEMO_DIR/workspace"
```

The serve command runs until its MCP client closes standard input. Configure the client with the compiled example executable, `serve` and the absolute workspace path. The server's standard output contains only protocol messages. All loaded documents are readable by that client.

Import accepts 1..256 visible, regular UTF-8 `.md` files, at most 4 MiB per file and 32 MiB total, with directory depth at most 32. Symlink sources are rejected. It preserves supplied bytes and labels positions as supplied text without inventing physical page numbers. The workspace JSON is bounded to 64 MiB, checksummed and validated on restoration. The checksum detects accidental changes; a writer who can replace both payload and checksum can supply new internally consistent data. Input documents can change later while an older saved workspace deliberately retains its original revision.

The output directory must be new. Complete JSON is published without replacing an existing workspace file. A write failure can leave the newly created directory without a usable workspace; choose another new path after resolving the error. Query work has a cooperative two-second deadline. Original documents are not rewritten.

## Measurable acceptance criteria

The versioned policy is [reference-v1-policy.json](evaluation/reference-v1-policy.json). Its thresholds are fixed before running candidate retrieval configurations.

| Area | Acceptance criterion |
| --- | --- |
| Evidence integrity | Every delivered span resolves to identical bytes in its registered snapshot. No stale revision or collection mismatch is accepted. |
| Bounds | At most six hits, 2,000 bytes per excerpt, 12,000 excerpt bytes total and 64 KiB complete JSON. |
| Failure behavior | Invalid files, changed snapshots and interrupted work produce errors. An empty search reports no evidence; a nonempty search leaves answerability unassessed. |
| Core retrieval budget | Warm p95 at most 25 ms on the frozen reference corpus; complete query JSON included, model inference excluded and reported separately. |
| Resource budget | Lexical worker peak RSS at most 512 MiB; construction at most 5 seconds for the reference corpus. These budgets describe this workload and recorded host. |
| Quality objective | Reference nDCG@10 at least 0.75 and reference Hit@6 at least 0.80 on answerable evaluation questions. Failure remains visible. |
| New profile recommendation | At least +0.03 reference nDCG@10 over literal BM25 on development and evaluation splits, Hit@6 loss no greater than 0.02, and the profile's resource budget met. |
| Refactoring | Identical ordered IDs, score bits and matched terms against the published baseline for both BM25 and TF-IDF; no reproducible p95 or RSS increase above 10% in paired controls. |
| Optional model profiles | Warm retrieval p95 at most 250 ms excluding external query embedding; reranking p95 at most 1,000 ms; full worker RSS at most 4 GiB. Model identity, setup and added query costs are separate measurements. |

## Frozen dataset

[reference-v1](../tests/fixtures/reference-v1/) contains 41 source documents copied byte for byte from published commit `6031fe5f5fda4727e13d89497aa238797cf6474f`, with per-file SHA-256 identities. Source licensing remains the repository's MIT license. It uses 150 newly authored questions: 90 development and 60 evaluation, with 20% intentionally unanswerable in each split. Each split has equal German and English counts. Twelve source families supply development questions and eight different families supply evaluation questions. Other documents remain searchable background material.

Every question records its origin. These are assistant-authored reference questions, not historical customer traffic or independent human judgments. No genuine user questions have been supplied for this corpus. That adoption evidence remains a separate future input; it does not prevent reproducing the reference evaluation.

Answerable cases identify an exact supporting quote in a frozen source. Judgments include passages containing that quote. Other passages can also support an answer, so reported nDCG and Hit@6 measure the known references. They are not exhaustive relevance recall or answer accuracy. Unanswerable questions have no positive judgment; they are excluded from the answerable nDCG mean and their returned-candidate rate is reported separately. Retrieving a passage is not an assertion that an unsupported question is answerable.

The evaluation split is frozen before candidate execution. Configuration selection uses development results only. Evaluation results are opened after that selection and are not used to retune version one. A changed question, source, judgment or budget requires a new dataset or policy identity. The same assistant authored the cases and implements the evaluation, so this is a reserved evaluation split, not an independently blinded assessment.

## Fair comparison

Literal AIRust BM25 is the compatibility baseline. Compare SQLite FTS5 using AIRust's pretokenized terms, then separately compare explicit lexical question preparation, coverage ranking and German text analysis. Keep original text, source spans, collection scope, candidate limit and output budgets equal. Preserve every per-question loss and every cost.

Optional hybrid comparisons use the same identified vectors, exact cosine retrieval and RRF parameters in both engines. Reranking receives only the actual first-stage candidate prefix. Missing model assets yield an unavailable profile, never a fabricated quality result. Quality gates and metrics reuse AIRust's existing modules.

Warm retrieval timing excludes index construction. Native inference, externally computed embeddings and serialization have explicit timing boundaries. Resource observations identify the worker, host, compiler, build profile and repetitions. Reports keep failed targets and slower observations and do not infer a universal speed, token-saving or production-capacity advantage.

## Work program

The verified release baseline and its evidence remain in [the 0.5.2 closeout record](validation-0.5.2/closeout.json). Subsequent work is tracked in [WORK-PROGRAM.json](WORK-PROGRAM.json): freeze the dataset and policy, implement the complete example, evaluate profiles, simplify onboarding, isolate complex index internals, and document the platform and PDF maintenance contracts. Each stage has a separate local commit and validation record.
