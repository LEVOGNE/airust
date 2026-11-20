# Reproduce AIRust 0.3.5 checks

Run `./test.sh` with Rust 1.92+ and Python 3.11+. Formatting is scoped to the owned `airust` package. The maintained optional backend is unchanged at 0.3.4; `python3 scripts/check_pdf_backend.py` checks source hashes, original notices and the small patch. Its own ten regressions run separately in CI.

The independent consumer.rs/consumer.toml application pins AIRust exactly to 0.3.5 and runs on Rust 1.89. Use a path override before publication, then registry resolution without that override. Minimum Rust 1.89/1.92, patch API checks against 0.3.4, fresh audits, clean packaging, extracted tests, dry-run and exact-commit Linux CI are release gates. Final registry bytes, consumer source, VCS record, tag and release assets are verified after publication.

The early-budget regression constructs its own two-page PDF. Page extraction must report its cumulative raw-text failure; conversion must first report the earlier chunk-count or overlap-expanded chunk-byte failure on the first page. The test failed against 0.3.4 and passes with the new pipeline. It does not use a fragile wall-clock threshold.

The ignored local-lab/pdf-stability directory retains originals, authored fixtures, old/new executables, paired timings, independently extracted references, canonical knowledge and raw mutation results. Only aggregate evidence is public. The first candidate's RAM regression is retained separately and was corrected before publication. No Docker run, exhaustive grammar fuzzing, security certification or application integration is claimed. See [pipeline/results/limits](../PDF-IMPORT-PIPELINE.md) and [public measurements](../measurements/0.3.5/).
