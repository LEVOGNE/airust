#![no_main]
#![forbid(unsafe_code)]

// Exercise the production framing/serde implementation without starting a
// subprocess, opening caller paths or loading a native runtime/model.
#[allow(dead_code)]
#[path = "../../src/reranking/process/protocol.rs"]
mod protocol;

use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let maximum = data.first().copied().unwrap_or(0) as usize * 16;
    let mut reader = Cursor::new(data);
    if let Ok(Some(frame)) = protocol::read_frame(&mut reader, maximum) {
        assert!(frame.len() <= maximum);
        assert_eq!(frame.last(), Some(&b'\n'));
        let _ = serde_json::from_slice::<protocol::Init>(&frame);
        let _ = serde_json::from_slice::<protocol::Request>(&frame);
        if let Ok(reply) = serde_json::from_slice::<protocol::Reply>(&frame) {
            let encoded = serde_json::to_vec(&reply).unwrap();
            let mut wire = Vec::new();
            protocol::write_frame(&mut wire, &encoded).unwrap();
            assert_eq!(wire.len(), encoded.len() + 1);
            assert_eq!(wire.last(), Some(&b'\n'));
        }
    }
    assert!(reader.position() <= maximum as u64 + 1);
    let diagnostic = protocol::bounded_message(String::from_utf8_lossy(data));
    assert!(diagnostic.len() <= 2048);
});
