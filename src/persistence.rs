use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

struct PendingFile(PathBuf);

impl Drop for PendingFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    atomic_write_with(path, true, true, |file| file.write_all(bytes))
}

#[cfg(any(feature = "pdf", test))]
pub(crate) fn atomic_write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    atomic_write_with(path, false, false, |file| file.write_all(bytes))
}

#[cfg(feature = "cache")]
pub(crate) fn atomic_write_private_with(
    path: &Path,
    write_content: impl FnOnce(&mut fs::File) -> io::Result<()>,
) -> io::Result<()> {
    atomic_write_with(path, false, true, write_content)
}

fn atomic_write_with(
    path: &Path,
    preserve_permissions: bool,
    replace: bool,
    write_content: impl FnOnce(&mut fs::File) -> io::Result<()>,
) -> io::Result<()> {
    let previous = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "refusing to replace a symbolic link",
            ));
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "destination must be a regular file",
            ));
        }
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if previous.is_some() && !replace {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "destination already exists",
        ));
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for _ in 0..32 {
        let temporary = parent.join(format!(
            ".airust-{}-{timestamp}-{}.tmp",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = match options.open(&temporary) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let pending = PendingFile(temporary);
        write_content(&mut file)?;
        if let Some(metadata) = &previous.filter(|_| preserve_permissions) {
            file.set_permissions(metadata.permissions())?;
        }
        file.sync_all()?;
        drop(file);
        if replace {
            fs::rename(&pending.0, path)?;
        } else {
            fs::hard_link(&pending.0, path)?;
        }
        drop(pending);
        #[cfg(unix)]
        fs::File::open(parent)?.sync_all()?;
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate an exclusive temporary file",
    ))
}

#[cfg(test)]
mod create_new_tests {
    use super::*;

    #[test]
    fn exclusive_publication_preserves_existing_bytes_and_cleans_staging() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("output");
        atomic_write_new(&path, b"complete first version").unwrap();
        assert_eq!(
            atomic_write_new(&path, b"replacement").unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(&path).unwrap(), b"complete first version");
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[test]
    fn concurrent_exclusive_writers_publish_exactly_one_complete_file() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("output");
        let barrier = std::sync::Barrier::new(8);
        let results = std::thread::scope(|scope| {
            (0..8)
                .map(|i| {
                    let path = &path;
                    let barrier = &barrier;
                    scope.spawn(move || {
                        barrier.wait();
                        atomic_write_new(path, &vec![i; 65536])
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|thread| thread.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
        assert!(results
            .iter()
            .filter_map(|r| r.as_ref().err())
            .all(|e| e.kind() == io::ErrorKind::AlreadyExists));
        let bytes = fs::read(&path).unwrap();
        assert_eq!(bytes.len(), 65536);
        assert!(bytes.iter().all(|b| *b == bytes[0]));
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    }
}

#[cfg(all(test, feature = "cache"))]
mod tests {
    use super::*;

    #[test]
    fn interrupted_stream_keeps_previous_file_and_removes_temporary_output() {
        let root = tempfile::tempdir().unwrap();
        let output = root.path().join("cache.bin");
        atomic_write(&output, b"previous complete snapshot").unwrap();
        let error = atomic_write_private_with(&output, |file| {
            file.write_all(b"partial new content")?;
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "simulated interrupted write",
            ))
        });
        assert!(error.is_err());
        assert_eq!(fs::read(&output).unwrap(), b"previous complete snapshot");
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
        let absent = root.path().join("absent.bin");
        assert!(atomic_write_private_with(&absent, |file| {
            file.write_all(b"half")?;
            Err(io::Error::other("write stopped"))
        })
        .is_err());
        assert!(!absent.exists());
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn killing_a_writer_before_rename_preserves_the_published_snapshot() {
        use std::{
            process::{Child, Command, Stdio},
            thread,
            time::{Duration, Instant},
        };
        struct KillOnDrop(Child);
        impl Drop for KillOnDrop {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let root = tempfile::tempdir().unwrap();
        let output = root.path().join("committed.bin");
        atomic_write(&output, b"committed snapshot").unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "persistence::tests::crash_writer_child",
                "--nocapture",
            ])
            .env("AIRUST_TEST_CRASH_OUTPUT", &output)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut child = KillOnDrop(child);
        let marker = output.with_extension("ready");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !marker.exists() && Instant::now() < deadline {
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "writer exited before its checkpoint"
            );
            thread::sleep(Duration::from_millis(5));
        }
        assert!(marker.exists(), "writer checkpoint timeout");
        child.0.kill().unwrap();
        assert!(!child.0.wait().unwrap().success());
        assert_eq!(fs::read(&output).unwrap(), b"committed snapshot");
        assert!(fs::read_dir(root.path())
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().starts_with(".airust-")));
        atomic_write_private_with(&output, |file| {
            file.write_all(b"complete recovery snapshot")
        })
        .unwrap();
        assert_eq!(fs::read(&output).unwrap(), b"complete recovery snapshot");
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "launched only by the parent crash-recovery test"]
    fn crash_writer_child() {
        let Ok(output) = std::env::var("AIRUST_TEST_CRASH_OUTPUT") else {
            return;
        };
        let output = PathBuf::from(output);
        atomic_write_private_with(&output, |file| {
            file.write_all(b"uncommitted half snapshot")?;
            file.sync_all()?;
            fs::write(output.with_extension("ready"), b"written before rename")?;
            loop {
                std::thread::park();
            }
        })
        .unwrap();
    }
}
