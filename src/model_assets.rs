//! Verified native assets without a native runtime dependency. Kept private so
//! model adapters own their manifests, recipe framing and publication policy.
use crate::input::BoundedFile;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::{self, Write},
    path::Path,
};

#[derive(Deserialize)]
pub(crate) struct Asset {
    pub(crate) bytes: u64,
    pub(crate) sha256: String,
}

fn open(path: &Path, expected: &Asset) -> io::Result<BoundedFile> {
    let file =
        BoundedFile::open(path, expected.bytes).map_err(crate::input::InputError::into_io)?;
    if file.len() != expected.bytes {
        return Err(invalid("local model asset size mismatch"));
    }
    Ok(file)
}

pub(crate) fn read(path: &Path, expected: &Asset) -> io::Result<Vec<u8>> {
    let bytes = open(path, expected)?
        .read()
        .map_err(crate::input::InputError::into_io)?;
    if format!("{:x}", Sha256::digest(&bytes)) != expected.sha256 {
        return Err(invalid("local model asset checksum mismatch"));
    }
    Ok(bytes)
}

pub(crate) fn snapshot(
    path: &Path,
    expected: &Asset,
    directory: &Path,
    mut consume: impl FnMut(&[u8]),
) -> io::Result<tempfile::NamedTempFile> {
    let mut input = open(path, expected)?;
    let mut snapshot = tempfile::Builder::new()
        .prefix("airust-rerank-")
        .suffix(".onnx")
        .tempfile_in(directory)?;
    let mut checksum = Sha256::new();
    input
        .read_chunks(|chunk| {
            snapshot.write_all(chunk)?;
            checksum.update(chunk);
            consume(chunk);
            Ok(())
        })
        .map_err(crate::input::InputError::into_io)?;
    if format!("{:x}", checksum.finalize()) != expected.sha256 {
        return Err(invalid("local model asset checksum mismatch"));
    }
    snapshot.flush()?;
    Ok(snapshot)
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_sizes_and_checksums_are_checked_without_native_model_loading() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model");
        let expected = Asset {
            bytes: 3,
            sha256: format!("{:x}", Sha256::digest(b"abc")),
        };
        assert!(read(&path, &expected).is_err());
        std::fs::write(&path, b"ab").unwrap();
        assert!(read(&path, &expected).is_err());
        std::fs::write(&path, b"abd").unwrap();
        assert!(read(&path, &expected).is_err());
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(read(&path, &expected).unwrap(), b"abc");
    }

    #[test]
    fn verified_snapshot_streams_exact_bytes_and_cleans_up_on_success_and_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model");
        // Cross the private reader's chunk boundary, including a short final chunk.
        let bytes = vec![0xA7; if cfg!(miri) { 71 } else { 65_543 }];
        std::fs::write(&path, &bytes).unwrap();
        let expected = Asset {
            bytes: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(&bytes)),
        };
        let mut hash = Sha256::new();
        let file = snapshot(&path, &expected, dir.path(), |chunk| {
            hash.update(chunk);
        })
        .unwrap();
        let copy = file.path().to_path_buf();
        assert_eq!(Sha256::digest(&bytes), hash.finalize());
        assert_eq!(std::fs::read(&copy).unwrap(), bytes);
        std::fs::write(&path, vec![0; bytes.len()]).unwrap();
        assert_eq!(std::fs::read(&copy).unwrap(), bytes);
        drop(file);
        assert!(!copy.exists());
        assert!(snapshot(&path, &expected, dir.path(), |_| {}).is_err());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
        assert_eq!(std::fs::read(&path).unwrap(), vec![0; bytes.len()]);
    }
}
