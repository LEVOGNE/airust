#![no_main]

#[path = "../../tests/support/fuzz_lexical.rs"]
mod fuzz_lexical;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    fuzz_lexical::LexicalCampaign::exercise(data);
});
