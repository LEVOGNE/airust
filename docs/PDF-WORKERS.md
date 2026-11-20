# Supervised PDF conversion

AIRust 0.3.4 retains this API and optimizes the previously slow parser case; the older timings below identify the 0.3.3 checks. See [current parser results](PDF-PARSER.md).

AIRust 0.3.3 adds an optional supervised process for PDF conversion. Use it when a parser call may take too long. This is a stability boundary around the existing importer; the knowledge engine and ranking APIs are unchanged.

## Run a bounded job

```bash
cargo run --release --locked --features cli,pdf --bin pdf2kb -- INPUT.pdf OUTPUT.json --timeout-seconds 60 --max-file-mib 64 --max-pages 3000 --max-text-mib 64 --max-chunks 50000 --max-result-mib 192
```

For the installed converter:

```bash
cargo install airust --version 0.3.5 --locked --features cli,pdf --bin pdf2kb
pdf2kb INPUT.pdf OUTPUT.json --timeout-seconds 60
```

`--timeout-seconds` accepts a positive whole number. It enables the [bounded importer defaults](PDF-LIMITS.md) and starts a child copy of the same converter. Explicit import limits override the corresponding defaults. `--max-result-mib` bounds the JSON result accepted by the parent; its default is 256 MiB and it requires a timeout. Invalid, zero and overflowing settings fail before conversion.

The child receives the exact chunk, weight, cleanup and metadata settings. Paths are passed as operating-system arguments after `--`, so they cannot become command options. No shell is used in the production worker.

## What happens to files

The child writes its complete result inside an exclusive temporary directory. On Unix this directory is private to the user. It never receives the final destination path. The parent waits for success, checks the result-file byte budget, parses the nonempty training array and validates its examples. It then publishes through the same atomic save methods used by ordinary conversion.

A worker conversion error, parser failure, failed result validation or worker deadline leaves the final destination untouched before publication. Existing output still requires `--overwrite`. Without that flag, a destination created while the job runs remains protected by exclusive publication. Temporary worker files are cleaned up on normal success/error paths; the successful final knowledge remains at the requested output path.

Worker stdout and stderr are drained concurrently with a limit of 256 KiB each. Capture errors or excess diagnostics stop the job. Bounded diagnostics from a failed child are returned as an ordinary conversion error; the parent escapes controls before displaying them. `--verbose` forwards bounded raw/cleaned previews after success and displays only the final publication message for the real output.

## Progress and time

Console messages are best-effort: a closed stdout/stderr connection does not panic or change knowledge-file outcomes. Actual knowledge-file I/O errors still propagate. The parent reports when the worker starts, reports real elapsed time every five seconds while it runs, and reports when output is validated. These are lifecycle messages, not a predicted completion percentage or an estimated answer time.

The monotonic budget covers worker setup, startup and execution. The supervisor polls about every ten milliseconds. At expiry it kills its owned child and waits for the exit status before returning a timeout error. A guard also stops and waits on error paths. The Rust [Child contract](https://doc.rust-lang.org/std/process/struct.Child.html) explains why explicit waiting is needed.

This is not a hard real-time deadline for the whole command. Argument/filesystem checks before the worker, scheduling delays, console I/O, process termination, parent result parsing, atomic publication and final synchronization can add time. A result that is not observed before expiry is rejected conservatively. The existing synchronous `PdfLoader` and its library methods remain synchronous; `WorkControl` does not preempt a blocking native PDF call.

## Boundaries

A separate process contains a child's normal failure or abort; it does not provide an operating-system security sandbox, a hard RAM/CPU/disk quota, hostile-code isolation or guaranteed cleanup after the parent is forcibly killed. System-wide out-of-memory conditions can still affect the parent. A service handling untrusted files should add its own operating-system isolation and resource policy. Original file authorization and source revisions remain the application's responsibility.

Result-byte limits bound what the parent reads. They do not impose a hard disk quota while the child is creating its private JSON. Parser memory can grow inside a single native call before a library text limit is checked. Keep meaningful input, page, text, chunk and job limits rather than treating any one flag as a complete resource policy.

No OCR, embedding model, HTTP server, database, DenkMaps adapter or MCP integration is added. The default library remains unchanged; the explicit CLI feature now also enables the existing optional temporary-file dependency.

## Executed checks

The real 494-page slow case returns a one-second deadline error in approximately 1.03 seconds, with its previous destination bytes preserved. With a 90-second budget, all fifteen PDFs (8,960 pages) import successfully and produce 33,708 chunks with the same checked text and provenance as the retained baseline. The slow first-page parser path itself is unchanged.

The supervised malformed-input campaign covers six authored files and 128 deterministic byte mutations. A further fixed-seed campaign checks 4,096 variants of the owned two-page fixture; it is repeated after the closed-console fix. Seventeen cases encounter the already known malformed-header unwinding panic in the PDF dependency and return controlled errors rather than partial knowledge. Every failure preserves previous output. An additional owned-worker SIGKILL probe confirms ordinary parent failure, worker removal and unchanged previous output. This simulates abrupt worker loss; it is not a system-wide OOM test. Unit/integration checks cover process reaping, nonzero child status, bounded diagnostics, invalid results, CLI settings, configuration forwarding, oversized JSON, concurrent writers and platform path bytes. Filesystems that reject raw non-Unicode filenames exercise clean rejection; exact byte-path creation is tested where supported.

A separate real-PDF index soak uses unchanged 0.3.2 engine sources. Each of two modes performs 20,000 operation attempts, 9,920 exact ranked-ID/score comparisons against rebuilt indexes, parallel reads and selected clone/JSON-roundtrip checks. Both retain identical final indexes. The lab's peak RSS includes all comparison copies and is not a standalone engine memory claim. See [aggregate measurements](measurements/0.3.3/).

Keep the ignored `local-lab/pdf-stability` directory to retain knowledge, reference text, indexes, fixtures, raw measurements, binaries and frozen release sources. Canonical source indexes remain separate from deliberately edited soak copies and malformed-input evidence; reuse the canonical indexes for domain knowledge. See [reuse instructions](PDF-STABILITY.md). Public documentation contains aggregates; private document text and query strings are not redistributed.
