use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub struct SeedManifest;

impl SeedManifest {
    pub fn replay(kind: &str, mut exercise: impl FnMut(&[u8])) {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fuzz-seeds/v0.5.2");
        let manifest = std::fs::read_to_string(root.join("MANIFEST.txt")).expect("seed manifest");
        let mut count = 0usize;
        let prefix = format!("{kind} ");
        for line in manifest.lines() {
            let Some(rest) = line.strip_prefix(&prefix) else {
                continue;
            };
            let mut parts = rest.split_whitespace();
            let expected = parts.next().expect("seed hash");
            let name = parts.next().expect("seed name");
            assert!(parts.next().is_none(), "{name}");
            assert!(
                !name.contains('/') && !name.contains('\\') && name != "." && name != "..",
                "{name}"
            );
            let bytes =
                std::fs::read(root.join(kind).join(name)).unwrap_or_else(|_| panic!("{name}"));
            assert_eq!(Self::hex(&Sha256::digest(&bytes)), expected, "{name}");
            exercise(&bytes);
            count += 1;
        }
        assert!(count > 0, "{kind}");
    }

    fn hex(bytes: &[u8]) -> String {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut text = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            text.push(DIGITS[(byte >> 4) as usize] as char);
            text.push(DIGITS[(byte & 0xf) as usize] as char);
        }
        text
    }
}
