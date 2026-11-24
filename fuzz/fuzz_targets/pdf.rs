#![no_main]

#[path = "../../tests/support/fuzz_pdf.rs"]
mod fuzz_pdf;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    fuzz_pdf::PdfCampaign::exercise(data);
});
