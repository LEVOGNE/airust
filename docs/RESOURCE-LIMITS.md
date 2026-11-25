# Resource ownership and small local applications

The default library has no model, server, database or network dependency. A large optional model is a different resource choice. These contracts keep ownership and rejection explicit; they do not impose an operating-system RAM quota.

## Training data snapshots

AIRust 0.6.1 stores `KnowledgeBase` training examples in a private shared `Arc<Vec<TrainingExample>>`. `clone()` and `from_embedded()` retain existing examples instead of copying every input, response and metadata value. Public types, borrowed slices, response formats, order and save format remain unchanged. A clone still owns its logical value: the first mutation of shared examples copies the full collection before changing it. Subsequent mutations of a uniquely owned collection do not copy it again. Removing an invalid index or merging an empty base does not detach shared examples.

This helps read-only snapshots, retained agent configurations and embedded-data reuse. It deliberately does not make many independent writers free: if every snapshot changes, each eventually needs its own examples. First-write latency can increase; keep it off a latency-critical path when retaining large readers. A saved file path is still cloned separately. An empty base also has a small reference-counted allocation. Retained readers keep their data alive; this is required ownership, not an automatic eviction cache.

The paired 6000-example/24-read-only-snapshot control observed worst worker RSS of 466026496 bytes for released 0.6.0 and 20955136 bytes for the candidate. With eight independently edited snapshots, both were about 169 MB. Every example signature matched. This is a declared synthetic training workload, not a search-index speedup, general memory ratio or actual old-device certification. [Measurements](measurements/profile-practical-0.6.0/README.md) retain all scenarios and first-write costs.

## Explicit file bounds

`KnowledgeBase::load_bounded(path, max_bytes)` checks that an application-owned path names a regular file, checks its size before allocating, reserves the known size fallibly, and rejects growth/shrinkage detected during reading. It validates all training weights before returning a new base. Zero allows only a zero-byte file, which is not a valid JSON training array. `load()` retains its compatible unbounded trusted-file behavior; choose the bounded entry point for caller-supplied files.

A byte cap bounds input bytes, not the total heap needed for the JSON buffer, decoded strings, records, index vocabulary and retained snapshots. Pick a small explicit cap. File paths and their directories must be controlled by the calling application. Static directories/devices/FIFOs are rejected; this is not protection against a hostile process concurrently replacing paths or modifying an asset with the same length. Symlinks to regular files remain usable. Save/atomic-publication contracts and original files are preserved on validation or output failure.

MCP startup file loading uses the same private regular-file reader while retaining `Configuration`/`Data`/`Io`/`Json` error categories. The E5 adapter now caps model bytes at 512 MiB, tokenizer bytes at 64 MiB and each of the three configuration files at 1 MiB before native initialization. Those caps do not promise the native runtime fits in that amount of RAM. Raw-byte/name/length recipe framing remains unchanged.

## Retained rejection diagnostics

A rejected input can outlive its request through a retained error. An invalid-UTF-8 error therefore contains the small UTF-8 position error, rather than a `FromUtf8Error` that also owns the rejected byte vector. Large serde JSON **data** diagnostics at restoration/import boundaries are clipped to at most 2048 UTF-8 bytes before publication. The diagnostic preserves a bounded beginning and ending, the expected-type explanation, and the original JSON category, line and column. Ordinary small errors and syntax/EOF/I/O error objects remain unchanged. Callers must use typed categories and positions rather than parse an entire diagnostic string as a stable data format.

This bounds the **returned diagnostic**. Serde can still temporarily allocate a formatted invalid string while parsing, and the caller's input buffer is still subject to its separate byte cap. It does not impose a total-process quota or sanitize arbitrary application errors. Native providers and arbitrary application backends have their own errors and budgets.

Forty-five separate controls retained 128 rejected 2-MiB inputs per worker across released 0.6.0, a deliberately retained pre-fix candidate and the fixed candidate. Large JSON error peaks fell from approximately 281 MB to 9.2–11.4 MB while all categories, positions and original files matched. The pre-fix candidate also accidentally retained invalid UTF-8 payloads (276 MB); the published baseline and fixed candidate were about 9 MB. That candidate regression was caught and fixed before publication, and its samples remain visible in [the evidence](measurements/profile-practical-0.6.0/README.md). These generated negative controls do not prove the absence of every allocation spike.

## A model-free bounded stdio host

Build the source candidate with only `mcp`:

```bash
cargo build --locked --release --no-default-features --features mcp --example bounded_mcp
"${CARGO_TARGET_DIR:-target}/release/examples/bounded_mcp" /path/to/entries.json --check
"${CARGO_TARGET_DIR:-target}/release/examples/bounded_mcp" /path/to/entries.json
```

The input is the existing JSON array of `KnowledgeEntry` records, for example:

```json
[{"id":"ownership","text":"Rust ownership manages memory safely."}]
```

`--check` reports loaded counts and worker admission without starting stdio. Normal execution serves the five existing read-only MCP tools. It uses no embedding service, answer model, PDF parser or model weights. MCP clients receive only the loaded data; they do not supply host file paths or alter these startup settings. These application limits are an example policy, not new global defaults:

| Resource | Example bound |
| --- | ---: |
| Entries / total ID, text and metadata bytes | 2048 / 2 MiB |
| Input JSON / individual text | 4 MiB / 4096 bytes |
| ID / metadata / query | 256 / 256 / 1024 bytes |
| Results / per-excerpt / complete tool output | 6 / 1024 bytes / 16 KiB |
| Concurrent CPU work / pending requests | 1 / 4 |
| Default / maximum request budget | 2 / 5 seconds |
| Frame / inbound rate / session input | 16 KiB / 128 KiB per second / 64 MiB |
| Messages per second / requests per session | 64 / 50000 |

The Tokio runtime has one async driver and capacity for three blocking tasks: stdin, stdout/progress and the single admitted CPU worker. On macOS/Windows, pending Tokio stdin reads occupy blocking capacity. Restricting that pool to one caused a reproduced response-starvation failure during development; the retained stdio/recovery test caught it. `max_concurrent` limits admitted library work, not all application threads or other tasks sharing a caller-owned runtime. Cooperative time/cancellation checks do not interrupt an arbitrary native call or guarantee a hard wall-clock deadline.

The existing SDK deliberately discards a malformed or oversized individual frame and permits the next valid request to recover. This is different from exceeding the session's raw-byte or frame-rate budgets, which closes the session. The small-host integration tests keep the client pipe open for both cases: valid requests recover after individual discarded frames; actual quota violations exit unsuccessfully without waiting for client EOF. An initial probe incorrectly required every malformed frame to terminate the process. That failed harness expectation and the rejected shutdown experiment are retained; production code was restored rather than changing the established recovery contract.

Run the declared synthetic resource probes separately from compilers, Miri, fuzzing or model work:

```bash
python3 -B scripts/benchmark_small_host.py \
  "${CARGO_TARGET_DIR:-target}/release/examples/bounded_mcp" /tmp/new-small-host-evidence
```

The script refuses an existing output directory, exercises Unicode, high-vocabulary and long-token datasets, alternates BM25/TF-IDF with unique queries, checks excerpts against originals and checks successful recovery after rejected oversized queries. It retains startup, first requests, warm raw samples, sampled post-warm RSS, peak child RSS and all failures. The declared observation budgets are 512 MiB worker peak and 32 MiB post-warm RSS growth over the finite run. The [final-source confirmation](measurements/configuration-0.6.1-final/README.md) passes all eight workers, including an explicit 393216-term stress case at about 212 MB peak. Its JSON framing is separate from the 2-MiB ID/text/metadata budget. Passing does not establish an unlimited corpus/session, whole-system usage, independent quality, energy efficiency or 1-GiB-device support. Actual result status belongs in [validation](validation-0.6.1/README.md).

## Optional native reranker

The ordinary MiniLM constructor remains memory-backed. `FastEmbedReranker::multilingual_minilm_v2_from_snapshot(model_directory, runtime_library, threads, snapshot_directory)` opts into a bounded 64-KiB streaming copy of the pinned model to an exclusively created private temporary file. Size/checksum verification covers the actual copied bytes; ONNX loads that copy, rather than re-opening an original that could have changed after hashing. Tokenizer files retain their pinned checks. The recipe and scores are not intentionally changed.

The model retains its file guard until native state has dropped. Initialization failures and normal drop remove the temporary copy; forced process termination can leave it behind. The application owns the directory and must provide at least the 470883696-byte model copy plus filesystem overhead in free space. There is no background deletion of user assets or arbitrary temporary files. Existing memory loading needs no writable snapshot directory.

For the reference profile, `reranker.model_snapshot_directory` enables this mode and resolves relative to its config file. Missing/null preserves memory mode. Offline `profile-check` resolves the setting but does not create the directory, verify disk space or load a model. Native FFI remains outside Miri. The separate pure-Rust file/checksum/snapshot helper can run in Miri without loading a native runtime.

Four paired native controls observed approximately 1.37–1.38 GB peak with the snapshot versus 1.66–1.70 GB memory-backed, with no changed raw score. Startup was sometimes slower. This is additional disk work for lower observed worker memory, not a guarantee across runtimes/platforms. Both native modes exceed a 1-GiB target here; BGE-M3 and the answer LLM have additional separate costs. Use the model-free core for the lightweight starting point and measure the actual device/workload before choosing native models.

## Repeated native loading and allocator retention

Cancellation is cooperative between native batches: an already executing ONNX call returns before the library can observe cancellation. The actual-asset lifecycle control cancels after one real batch, checks an already expired deadline, performs a fresh identical query and drops the scorer. Snapshot files were removed after all eight cycles and after a deliberately failed native constructor. These are native FFI controls, outside Miri.

The reference server constructs one configured scorer and reuses it across requests. Model initialization is not performed for each query. Applications that repeatedly construct/drop native providers must account for the lifecycle results separately; reusing an instance does not turn a failed repeated-load budget into a pass.

The default macOS allocator failed the frozen 64-MiB post-warm RSS-growth budget in two eight-cycle runs. After-drop RSS rose to about 2.36–2.43 GB. `vmmap` showed large empty malloc regions and fragmentation; `/usr/bin/leaks` reported zero unreachable malloc allocations. Reusable allocator pages and RSS are not equivalent to live Rust values. That diagnostic is evidence of allocator retention, not proof that every native leak has been excluded. Both failed controls are retained.

A separate child-process launch with `MallocSpaceEfficient=1`, using the same native binary and workload, passed that growth budget: the post-warm RSS range was 7602176 bytes and `vmmap` reported `26.5M` physical footprint after drop. Apple documents this flag in its [allocator source](https://github.com/apple-oss-distributions/libmalloc/blob/c49dafa25f1efe8607701ae6014a663ad2ee437f/src/malloc.c): it trades time for space and changes reclamation/cache policy. The installed OS and that source revision have separate identities. This library does not set environment variables, replace the allocator or call unsafe allocator APIs. A separate [same-final-binary request comparison](measurements/configuration-0.6.1-final/README.md#optional-macos-allocator-request-cost) passes the declared maximum 10% repeated p95-overhead gate (ratios 0.983 / 0.997), while retaining every slower query and startup/initial cost. Treat it as an optional macOS **application launch** experiment, not a library default or broad deployment recommendation. It does not bring active model inference below 1 GiB, apply to every OS, or turn the default failed runs into passes.
