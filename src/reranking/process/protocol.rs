use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Read, Write};

pub(super) const REQUEST_BYTES: usize = 16 * 1024 * 1024;
pub(super) const RESPONSE_BYTES: usize = 64 * 1024;
pub(super) const TEXT_BYTES: usize = 2 * 1024 * 1024;
pub(super) const DOCUMENTS: usize = 128;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Init {
    pub version: String,
    pub model_directory: std::path::PathBuf,
    pub runtime_library: std::path::PathBuf,
    pub threads: usize,
    pub snapshot_directory: std::path::PathBuf,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Request {
    pub id: u64,
    pub query: String,
    pub documents: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(super) enum Reply {
    Ready {
        version: String,
        pid: u32,
        recipe: String,
    },
    Scores {
        id: u64,
        scores: Vec<u64>,
    },
    Failure {
        id: u64,
        output_error: bool,
        message: String,
    },
}

pub(super) fn read_frame<R: BufRead>(
    reader: &mut R,
    maximum: usize,
) -> io::Result<Option<Vec<u8>>> {
    let mut frame = Vec::new();
    let count = (&mut *reader)
        .take(maximum as u64 + 1)
        .read_until(b'\n', &mut frame)?;
    if count == 0 {
        return Ok(None);
    }
    if count > maximum || frame.last() != Some(&b'\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid or oversized worker frame",
        ));
    }
    Ok(Some(frame))
}

pub(super) fn write_frame<W: Write>(writer: &mut W, frame: &[u8]) -> io::Result<()> {
    writer.write_all(frame)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

pub(super) fn bounded_message(message: impl std::fmt::Display) -> String {
    use std::fmt::Write;
    struct Diagnostic(String);
    impl Write for Diagnostic {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            let available = 2048 - self.0.len();
            let mut end = value.len().min(available);
            while !value.is_char_boundary(end) {
                end -= 1;
            }
            self.0.push_str(&value[..end]);
            if end < value.len() {
                return Err(std::fmt::Error);
            }
            Ok(())
        }
    }
    let mut output = Diagnostic(String::with_capacity(2048));
    // Stop formatting at the byte budget, before allocating a complete error.
    let _ = write!(&mut output, "{message}");
    output.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framing_bounds_partial_and_oversized_streams_without_consuming_the_next_frame() {
        let mut input = &b"one\ntwo\n"[..];
        assert_eq!(read_frame(&mut input, 4).unwrap().unwrap(), b"one\n");
        assert_eq!(read_frame(&mut input, 4).unwrap().unwrap(), b"two\n");
        assert!(read_frame(&mut input, 4).unwrap().is_none());
        assert!(read_frame(&mut &b"four\n"[..], 4).is_err());
        assert!(read_frame(&mut &b"one"[..], 4).is_err());
        assert!(read_frame(&mut &b"\n"[..], 0).is_err());
    }

    #[test]
    fn diagnostics_and_protocol_shape_are_bounded_and_strict() {
        let message = bounded_message("界".repeat(2000));
        assert!(message.len() <= 2048 && message.is_char_boundary(message.len()));
        assert!(serde_json::from_str::<Request>(
            r#"{"id":1,"query":"q","documents":[],"extra":true}"#
        )
        .is_err());
        let mut output = Vec::new();
        write_frame(&mut output, br#"{"id":1}"#).unwrap();
        assert_eq!(output.last(), Some(&b'\n'));
    }

    #[test]
    fn diagnostic_formatting_stops_at_its_budget() {
        struct Large<'a>(&'a std::cell::Cell<usize>);
        impl std::fmt::Display for Large<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for _ in 0..10_000 {
                    self.0.set(self.0.get() + 1);
                    f.write_str("界")?;
                }
                Ok(())
            }
        }
        let writes = std::cell::Cell::new(0);
        let message = bounded_message(Large(&writes));
        assert_eq!(message.len(), 2046);
        assert!(writes.get() < 700);
    }

    #[test]
    fn mutated_frames_remain_finite_and_wire_scores_preserve_raw_bits() {
        let rounds = if cfg!(miri) { 8 } else { 2048 };
        let mut state = 0x06120612u64;
        for _ in 0..rounds {
            let mut bytes = vec![0u8; 127];
            for byte in &mut bytes {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                *byte = state as u8;
            }
            let maximum = usize::from(bytes[0]) % 64;
            let mut input = std::io::Cursor::new(&bytes);
            if let Ok(Some(frame)) = read_frame(&mut input, maximum) {
                assert!(frame.len() <= maximum && frame.last() == Some(&b'\n'));
                let _ = serde_json::from_slice::<Reply>(&frame);
            }
            assert!(input.position() <= maximum as u64 + 1);
        }
        let bits = vec![
            (-0.0f64).to_bits(),
            f64::MIN_POSITIVE.to_bits(),
            1u64,
            f64::MAX.to_bits(),
        ];
        let encoded = serde_json::to_vec(&Reply::Scores {
            id: 1,
            scores: bits.clone(),
        })
        .unwrap();
        let Reply::Scores { id, scores } = serde_json::from_slice(&encoded).unwrap() else {
            panic!("shape")
        };
        assert_eq!(id, 1);
        assert_eq!(scores, bits);
    }
}
