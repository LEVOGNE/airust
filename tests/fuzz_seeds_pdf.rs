#![cfg(feature = "pdf")]

#[path = "support/fuzz_pdf.rs"]
mod fuzz_pdf;

#[path = "support/fuzz_manifest.rs"]
mod fuzz_manifest;

#[test]
fn versioned_pdf_seeds_match_the_manifest_and_replay() {
    fuzz_manifest::SeedManifest::replay("pdf", |bytes| {
        fuzz_pdf::PdfCampaign::exercise(bytes);
    });
}
