#![no_main]

#[path = "../../tests/support/fuzz_knowledge.rs"]
mod fuzz_knowledge;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    fuzz_knowledge::KnowledgeCampaign::exercise(data);
});
