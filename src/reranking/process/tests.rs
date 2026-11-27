use super::*;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
    time::Instant,
};

fn fixture(mode: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let worker = root.path().join("worker");
    let runtime = root.path().join("runtime");
    fs::write(&runtime, b"mock runtime, never loaded").unwrap();
    let script = format!(
        r#"#!/usr/bin/env python3
import json,os,sys,time,struct
mode={mode:?}
init=json.loads(sys.stdin.readline())
if mode=='startup_timeout':time.sleep(20)
if init.get('snapshot_directory'):
    open(os.path.join(init['snapshot_directory'],'owned-copy'),'wb').write(b'fixture')
if mode=='observe':
    observed={{'environment':sorted(os.environ),'runtime':init['runtime_library'],'runtime_bytes':open(init['runtime_library'],'rb').read().decode()}}
    open(os.path.join(init['model_directory'],'observed'),'w').write(json.dumps(observed))
def send(value):
    print(json.dumps(value),flush=True)
send({{'kind':'Ready','version':{version:?} if mode!='version' else 'wrong','pid':os.getpid(),'recipe':'fixture:v1'}})
if mode=='blocked_input':time.sleep(20)
for line in sys.stdin:
    request=json.loads(line)
    if mode=='timeout':time.sleep(20)
    if mode=='crash':os._exit(7)
    if mode=='oversize':print('x'*70000,flush=True);continue
    if mode=='malformed':print('{{',flush=True);continue
    if mode=='provider':send({{'kind':'Failure','id':request['id'],'output_error':False,'message':'x'*9000}});continue
    bits=struct.unpack('<Q',struct.pack('<d',-0.0))[0]
    if mode=='nonfinite':bits=0x7ff8000000000000
    scores=[bits for _ in request['documents']]
    if mode=='count':scores=[]
    send({{'kind':'Scores','id':request['id']+(1 if mode=='id' else 0),'scores':scores}})
if mode=='hang_exit':time.sleep(20)
open(os.path.join(init['model_directory'],'closed'),'w').write('yes')
"#,
        version = crate::VERSION
    );
    fs::write(&worker, script).unwrap();
    fs::set_permissions(&worker, fs::Permissions::from_mode(0o700)).unwrap();
    (root, worker, runtime)
}

fn gone(pid: u32) -> bool {
    !Command::new("/bin/kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success()
}

#[test]
fn graceful_drop_reaps_worker_preserves_originals_and_removes_only_private_snapshots() {
    let (root, worker, runtime) = fixture("good");
    let original = root.path().join("original");
    fs::write(&original, b"retain original").unwrap();
    let copies = root.path().join("copies");
    fs::create_dir(&copies).unwrap();
    let mut model = ProcessReranker::multilingual_minilm_v2_from_snapshot(
        ProcessOptions::new(worker),
        root.path(),
        &runtime,
        2,
        &copies,
    )
    .unwrap();
    let pid = model.worker_id();
    assert_eq!(fs::read_dir(&copies).unwrap().count(), 1);
    assert!(matches!(
        model.score(&"q".repeat(16385), &["d"]),
        Err(RerankError::Budget(_))
    ));
    let scores = model.score("q", &["a", "b"]).unwrap();
    assert_eq!(
        scores.iter().map(|f| f.to_bits()).collect::<Vec<_>>(),
        vec![(-0.0f64).to_bits(); 2]
    );
    let escaped = "\u{1}".repeat(64 * 1024);
    assert!(matches!(
        model.score("q", &[escaped.as_str(); 32]),
        Err(RerankError::Budget(_))
    ));
    // About twelve MiB on the wire exercises sixfold JSON escaping and pipe
    // backpressure, without loading a native model or exceeding text limits.
    let scores = model.score("q", &[escaped.as_str(); 31]).unwrap();
    assert_eq!(scores.len(), 31);
    assert!(scores.iter().all(|s| s.to_bits() == (-0.0f64).to_bits()));
    assert_eq!(model.score("q", &["recovery"]).unwrap().len(), 1);
    drop(model);
    assert!(gone(pid));
    assert!(root.path().join("closed").is_file());
    assert_eq!(fs::read_dir(&copies).unwrap().count(), 0);
    assert_eq!(fs::read(original).unwrap(), b"retain original");
    assert!(runtime.is_file());
}

#[test]
fn malformed_mismatched_and_nonfinite_worker_outputs_fail_closed_and_stop_the_worker() {
    for mode in [
        "oversize",
        "malformed",
        "id",
        "count",
        "nonfinite",
        "provider",
        "crash",
    ] {
        let (root, worker, runtime) = fixture(mode);
        let mut model = ProcessReranker::multilingual_minilm_v2_from_snapshot(
            ProcessOptions::new(worker),
            root.path(),
            runtime,
            2,
            root.path(),
        )
        .unwrap();
        let pid = model.worker_id();
        let error = model.score("q", &["d"]).unwrap_err();
        match mode {
            "count" => assert!(matches!(
                error,
                RerankError::ScoreCount {
                    expected: 1,
                    actual: 0
                }
            )),
            "nonfinite" => assert_eq!(error, RerankError::NonFiniteScore(1)),
            "provider" => {
                assert!(matches!(error, RerankError::Provider(ref message) if message.len()<=2048))
            }
            _ => assert!(matches!(
                error,
                RerankError::Output(_) | RerankError::Provider(_)
            )),
        }
        assert!(gone(pid), "{mode}");
        assert!(model.score("q", &["d"]).is_err());
    }
}

#[test]
fn timeout_and_noncooperative_exit_are_reaped_without_affecting_the_parent() {
    for mode in ["timeout", "blocked_input", "hang_exit"] {
        let (root, worker, runtime) = fixture(mode);
        let options = ProcessOptions::new(worker)
            .with_timeouts(Duration::from_secs(5), Duration::from_millis(100))
            .unwrap();
        let mut model = ProcessReranker::multilingual_minilm_v2_from_snapshot(
            options,
            root.path(),
            runtime,
            2,
            root.path(),
        )
        .unwrap();
        let pid = model.worker_id();
        let start = Instant::now();
        if mode != "hang_exit" {
            let document = "x".repeat(64 * 1024);
            assert!(matches!(
                model.score("q", &[document.as_str(); 31]),
                Err(RerankError::Provider(_))
            ));
        }
        drop(model);
        assert!(start.elapsed() < Duration::from_secs(5));
        assert!(gone(pid));
    }
}

#[test]
fn startup_identity_and_invalid_configuration_do_not_publish_a_model() {
    let (root, worker, runtime) = fixture("version");
    assert!(matches!(
        ProcessReranker::multilingual_minilm_v2_from_snapshot(
            ProcessOptions::new(worker),
            root.path(),
            runtime,
            2,
            root.path()
        ),
        Err(RerankError::Output(_))
    ));
    assert!(ProcessOptions::new("unused")
        .with_timeouts(Duration::ZERO, Duration::from_secs(1))
        .is_err());
    assert!(matches!(
        ProcessReranker::multilingual_minilm_v2_from_snapshot(
            ProcessOptions::new("unused"),
            "missing",
            "missing",
            65,
            "missing"
        ),
        Err(RerankError::Configuration(_))
    ));
}

#[test]
fn blocked_initialization_is_cancelled_and_its_private_directory_is_removed() {
    let (root, worker, runtime) = fixture("startup_timeout");
    let copies = root.path().join("copies");
    fs::create_dir(&copies).unwrap();
    let options = ProcessOptions::new(worker)
        .with_timeouts(Duration::from_millis(100), Duration::from_secs(1))
        .unwrap();
    let start = Instant::now();
    assert!(matches!(
        ProcessReranker::multilingual_minilm_v2_from_snapshot(
            options,
            root.path(),
            runtime,
            2,
            &copies
        ),
        Err(RerankError::Provider(_))
    ));
    assert!(start.elapsed() < Duration::from_secs(5));
    assert_eq!(fs::read_dir(copies).unwrap().count(), 0);
}

#[test]
fn explicit_recreation_after_crashes_reuses_the_parent_without_retaining_workers_or_copies() {
    let (root, worker, runtime) = fixture("crash");
    let script = fs::read_to_string(&worker).unwrap();
    let original = root.path().join("original");
    fs::write(&original, b"retain across failures").unwrap();
    let copies = root.path().join("copies");
    fs::create_dir(&copies).unwrap();
    for mode in ["crash", "good", "crash", "good"] {
        fs::write(
            &worker,
            script.replace("mode=\"crash\"", &format!("mode={mode:?}")),
        )
        .unwrap();
        let mut model = ProcessReranker::multilingual_minilm_v2_from_snapshot(
            ProcessOptions::new(&worker),
            root.path(),
            &runtime,
            2,
            &copies,
        )
        .unwrap();
        let pid = model.worker_id();
        let result = model.score("query", &["document"]);
        assert_eq!(result.is_ok(), mode == "good");
        if mode == "crash" {
            assert!(gone(pid));
            assert!(model.score("query", &["document"]).is_err());
        }
        drop(model);
        assert!(gone(pid));
        assert_eq!(fs::read_dir(&copies).unwrap().count(), 0);
        assert_eq!(fs::read(&original).unwrap(), b"retain across failures");
        assert!(runtime.is_file());
    }
}

#[test]
fn worker_environment_is_reduced_and_verified_runtime_copies_are_loaded() {
    let (root, worker, runtime) = fixture("observe");
    let copies = root.path().join("copies");
    fs::create_dir(&copies).unwrap();
    let bytes = fs::metadata(&runtime).unwrap().len();
    let digest = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(fs::read(&runtime).unwrap()))
    };
    let options = ProcessOptions::new(&worker)
        .with_runtime_identity(bytes, &digest)
        .unwrap();
    let model = ProcessReranker::multilingual_minilm_v2_from_snapshot(
        options,
        root.path(),
        &runtime,
        2,
        &copies,
    )
    .unwrap();
    let pid = model.worker_id();
    let observed: serde_json::Value =
        serde_json::from_slice(&fs::read(root.path().join("observed")).unwrap()).unwrap();
    for name in observed["environment"].as_array().unwrap() {
        let name = name.as_str().unwrap();
        assert!(
            transport::WORKER_ENVIRONMENT.contains(&name) || name.starts_with("__"),
            "worker inherited {name}"
        );
    }
    let loaded = PathBuf::from(observed["runtime"].as_str().unwrap());
    assert!(loaded.starts_with(copies.canonicalize().unwrap()));
    assert_ne!(loaded, runtime.canonicalize().unwrap());
    assert_eq!(observed["runtime_bytes"], "mock runtime, never loaded");
    drop(model);
    assert!(gone(pid));
    assert_eq!(fs::read_dir(&copies).unwrap().count(), 0);
    assert!(runtime.is_file());

    let wrong = ProcessOptions::new(&worker)
        .with_runtime_identity(bytes, &"0".repeat(64))
        .unwrap();
    let rejected = ProcessReranker::multilingual_minilm_v2_from_snapshot(
        wrong,
        root.path(),
        &runtime,
        2,
        &copies,
    );
    assert!(matches!(rejected, Err(RerankError::Provider(_))));
    assert_eq!(fs::read_dir(&copies).unwrap().count(), 0);
    for (size, hash) in [
        (0, digest.as_str()),
        (bytes, "ABC"),
        ((1 << 30) + 1, digest.as_str()),
    ] {
        assert!(matches!(
            ProcessOptions::new(&worker).with_runtime_identity(size, hash),
            Err(RerankError::Configuration(_))
        ));
    }
}
