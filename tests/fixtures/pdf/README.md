# Authored PDF regressions

These minimal test PDFs were generated for AIRust and contain no private document material. Blank, simple two-page, damaged and password-protected inputs exercise converter failures and provenance. The encrypted file's password is an arbitrary test value, not a credential for any service.

`invalid-header-utf8.pdf` is mutation 19 from the deterministic seed 20261120 applied to an authored minimal valid PDF. It reproduces a byte/character-boundary panic inside pdf_oxide 0.3.78. AIRust must return a normal conversion error, preserving existing output. A runtime-generated 500-page fixture in tests/support/pdf_fixture.rs exercises concurrent creation of a CLI output.


The owned `forms-depth-10.pdf`, `forms-depth-11.pdf` and `fallback-count-depth-50.pdf` fixtures contain explicit text witnesses inside nested Form XObjects or a page tree without a root Count. They reproduce complete supported extraction and rejection after work-limit exhaustion. They contain no third-party/private document text. Their source generation and structural inventory remain retained in the PDF stability lab; independent backend tests generate the same kinds of structures in Rust.
