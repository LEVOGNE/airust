//! Private file capability: regular inputs, checked lengths and bounded allocation.

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum InputError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("input must be a regular file")]
    NotRegular,
    #[error("input exceeds byte limit")]
    TooLarge,
    #[error("input changed length after opening")]
    Changed,
    #[error("input length exceeds addressable memory")]
    Length,
    #[error("cannot reserve input buffer: {0}")]
    Allocation(std::collections::TryReserveError),
}

impl InputError {
    pub(crate) fn into_io(self) -> io::Error {
        match self {
            Self::Io(error) => error,
            Self::Allocation(error) => io::Error::new(io::ErrorKind::OutOfMemory, error),
            other => io::Error::new(io::ErrorKind::InvalidData, other),
        }
    }
}

pub(crate) struct BoundedFile {
    file: File,
    length: u64,
}

impl BoundedFile {
    pub(crate) fn open(path: &Path, maximum: u64) -> Result<Self, InputError> {
        // Reject static FIFOs/devices before opening. This is not a sandbox against
        // another process replacing paths concurrently; callers own asset directories.
        let metadata = std::fs::metadata(path)?;
        if !metadata.is_file() {
            return Err(InputError::NotRegular);
        }
        if metadata.len() > maximum {
            return Err(InputError::TooLarge);
        }
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(InputError::NotRegular);
        }
        if metadata.len() > maximum {
            return Err(InputError::TooLarge);
        }
        Ok(Self {
            file,
            length: metadata.len(),
        })
    }

    pub(crate) fn len(&self) -> u64 {
        self.length
    }

    pub(crate) fn read(mut self) -> Result<Vec<u8>, InputError> {
        let length = allocation_length(self.len())?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(InputError::Allocation)?;
        self.read_chunks(|chunk| {
            bytes.extend_from_slice(chunk);
            Ok(())
        })?;
        Ok(bytes)
    }

    pub(crate) fn read_chunks(
        &mut self,
        mut consume: impl FnMut(&[u8]) -> io::Result<()>,
    ) -> Result<(), InputError> {
        let mut buffer = [0_u8; 64 * 1024];
        let mut total = 0_u64;
        loop {
            let count = match self.file.read(&mut buffer) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => result?,
            };
            if count == 0 {
                break;
            }
            total = total
                .checked_add(count as u64)
                .filter(|total| *total <= self.length)
                .ok_or(InputError::Changed)?;
            consume(&buffer[..count])?;
        }
        if total != self.length {
            return Err(InputError::Changed);
        }
        Ok(())
    }
}

fn allocation_length(length: u64) -> Result<usize, InputError> {
    usize::try_from(length)
        .ok()
        .filter(|length| *length <= isize::MAX as usize)
        .ok_or(InputError::Length)
}

const MAX_JSON_ERROR_BYTES: usize = 2048;
const TRUNCATED: &str = " ... [truncated] ... ";

/// Large serde data errors can own a formatted copy of an invalid input string.
/// Bound that diagnostic before returning it, preserving its category/position.
/// Small errors and syntax/EOF/IO categories retain their original error object.
pub(crate) fn json_error(error: serde_json::Error) -> serde_json::Error {
    use std::fmt::Write;
    if !error.is_data() {
        return error;
    }
    struct Count(usize);
    impl std::fmt::Write for Count {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0 = self.0.saturating_add(text.len());
            if self.0 > MAX_JSON_ERROR_BYTES {
                Err(std::fmt::Error)
            } else {
                Ok(())
            }
        }
    }
    if write!(&mut Count(0), "{error}").is_ok() {
        return error;
    }
    let head_limit = (MAX_JSON_ERROR_BYTES - TRUNCATED.len()) * 3 / 4;
    let tail_limit = MAX_JSON_ERROR_BYTES - TRUNCATED.len() - head_limit;
    struct Diagnostic {
        head: String,
        tail: String,
        head_limit: usize,
        tail_limit: usize,
        head_done: bool,
    }
    impl std::fmt::Write for Diagnostic {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            if !self.head_done {
                let mut end = text.len().min(self.head_limit - self.head.len());
                while !text.is_char_boundary(end) {
                    end -= 1;
                }
                self.head.push_str(&text[..end]);
                self.head_done = end < text.len() || self.head.len() == self.head_limit;
            }
            if text.len() >= self.tail_limit {
                let mut start = text.len() - self.tail_limit;
                while !text.is_char_boundary(start) {
                    start += 1;
                }
                self.tail.clear();
                self.tail.push_str(&text[start..]);
            } else {
                self.tail.push_str(text);
                if self.tail.len() > self.tail_limit {
                    let mut start = self.tail.len() - self.tail_limit;
                    while !self.tail.is_char_boundary(start) {
                        start += 1;
                    }
                    self.tail.drain(..start);
                }
            }
            Ok(())
        }
    }
    let mut message = Diagnostic {
        head: String::with_capacity(head_limit),
        tail: String::with_capacity(tail_limit),
        head_limit,
        tail_limit,
        head_done: false,
    };
    let _ = write!(&mut message, "{error}");
    // Keep the original display suffix, including line/column. serde_json's
    // public custom-error path restores those positions from that suffix.
    <serde_json::Error as serde::de::Error>::custom(format!(
        "{}{}{}",
        message.head, TRUNCATED, message.tail
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn allocation_lengths_and_failures_keep_resource_errors_distinct() {
        assert_eq!(allocation_length(0).unwrap(), 0);
        assert_eq!(allocation_length(65536).unwrap(), 65536);
        assert!(matches!(
            allocation_length(u64::MAX),
            Err(InputError::Length)
        ));
        let failure = Vec::<u8>::new().try_reserve_exact(usize::MAX).unwrap_err();
        assert_eq!(
            InputError::Allocation(failure).into_io().kind(),
            io::ErrorKind::OutOfMemory
        );
    }

    #[test]
    fn json_data_diagnostics_are_bounded_utf8_and_preserve_positions_and_categories() {
        let input = serde_json::to_string(&"Grüße 東京 🙂".repeat(300)).unwrap();
        let original = serde_json::from_str::<Vec<u8>>(&input).unwrap_err();
        let position = (original.line(), original.column(), original.classify());
        let bounded = json_error(original);
        assert_eq!(
            (bounded.line(), bounded.column(), bounded.classify()),
            position
        );
        let diagnostic = bounded.to_string();
        assert!(diagnostic.len() <= MAX_JSON_ERROR_BYTES);
        assert!(diagnostic.contains("[truncated]"));
        assert!(diagnostic.contains("expected a sequence"));
        for input in ["[", "[1,]", "false"] {
            let error = serde_json::from_str::<Vec<u8>>(input).unwrap_err();
            let message = error.to_string();
            let category = error.classify();
            let bounded = json_error(error);
            assert_eq!(bounded.to_string(), message);
            assert_eq!(bounded.classify(), category);
        }
        let error =
            serde_json::Error::io(io::Error::new(io::ErrorKind::PermissionDenied, "denied"));
        assert_eq!(
            json_error(error).io_error_kind(),
            Some(io::ErrorKind::PermissionDenied)
        );
    }

    #[test]
    fn exact_limit_empty_and_oversized_inputs() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        assert!(BoundedFile::open(file.path(), 0)
            .unwrap()
            .read()
            .unwrap()
            .is_empty());
        file.write_all(b"abc").unwrap();
        assert_eq!(
            BoundedFile::open(file.path(), 3).unwrap().read().unwrap(),
            b"abc"
        );
        assert!(BoundedFile::open(file.path(), 2).is_err());
        // A sparse oversized file must be rejected without reserving its payload.
        file.as_file().set_len(1_u64 << 40).unwrap();
        assert!(BoundedFile::open(file.path(), 1024).is_err());
    }

    #[test]
    fn rejects_growth_shrink_and_non_files() {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), b"abc").unwrap();
        let input = BoundedFile::open(file.path(), 3).unwrap();
        std::fs::write(file.path(), b"abcd").unwrap();
        assert!(input.read().is_err());
        let input = BoundedFile::open(file.path(), 4).unwrap();
        std::fs::write(file.path(), b"a").unwrap();
        assert!(input.read().is_err());
        let dir = tempfile::tempdir().unwrap();
        assert!(BoundedFile::open(dir.path(), 1024).is_err());
    }

    #[test]
    fn streams_exact_bytes_and_propagates_consumer_errors() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let bytes = vec![0xA7; 131_089];
        std::fs::write(file.path(), &bytes).unwrap();
        let mut input = BoundedFile::open(file.path(), bytes.len() as u64).unwrap();
        assert_eq!(input.len(), bytes.len() as u64);
        let mut actual = Vec::new();
        input
            .read_chunks(|chunk| {
                actual.extend_from_slice(chunk);
                Ok(())
            })
            .unwrap();
        assert_eq!(actual, bytes);
        let mut input = BoundedFile::open(file.path(), bytes.len() as u64).unwrap();
        let error = input
            .read_chunks(|_| Err(io::Error::other("destination failed")))
            .unwrap_err();
        assert_eq!(error.to_string(), "destination failed");
        assert_eq!(std::fs::read(file.path()).unwrap(), bytes);
    }

    #[cfg(unix)]
    #[test]
    fn follows_regular_symlinks_but_rejects_devices() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("original");
        let link = dir.path().join("link");
        std::fs::write(&path, b"safe").unwrap();
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert_eq!(
            BoundedFile::open(&link, 4).unwrap().read().unwrap(),
            b"safe"
        );
        assert!(BoundedFile::open(Path::new("/dev/null"), 4).is_err());
    }
}
