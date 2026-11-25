#[path = "support/fuzz_knowledge.rs"]
mod fuzz_knowledge;

#[test]
fn retained_training_snapshots_and_bounded_restore_match_owned_values() {
    #[cfg(miri)]
    let iterations = 8;
    #[cfg(not(miri))]
    let iterations = 128;
    let mut seed = 0x861A_CAFE_u64;
    for length in 0..iterations {
        let mut input = Vec::new();
        for _ in 0..length {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            input.push((seed >> 32) as u8);
        }
        fuzz_knowledge::KnowledgeCampaign::exercise(&input);
    }
}

#[test]
fn training_restore_covers_legacy_json_unicode_and_malformed_inputs() {
    for input in [
        b"[]".as_slice(),
        br#"[{"input":"a","output":"b","weight":1}]"#,
        br#"[{"input":"a","output":"b","weight":-1}]"#,
        br#"[{"input":"a","output":{"Text":"b"},"weight":null}]"#,
        br#"{"examples":[]}"#,
        "Straße 東京 🦀".as_bytes(),
        &[0xFF, 0, 0xA7],
    ] {
        fuzz_knowledge::KnowledgeCampaign::exercise(input);
    }
}
