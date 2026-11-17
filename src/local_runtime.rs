use ort::environment::Environment;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

pub(crate) fn environment(path: &Path) -> Result<Environment, String> {
    static SHARED: Mutex<Option<(PathBuf, Environment)>> = Mutex::new(None);
    let path = path.canonicalize().map_err(|error| error.to_string())?;
    let mut shared = SHARED
        .lock()
        .map_err(|_| "local model environment lock is poisoned".to_string())?;
    if let Some((loaded, environment)) = shared.as_ref() {
        if loaded != &path {
            return Err(
                "local models must share the same ONNX Runtime library in one process".into(),
            );
        }
        return Ok(environment.clone());
    }
    let environment = ort::init_from(&path)
        .map_err(|error| error.to_string())?
        .with_name("airust-local-models")
        .build()
        .map_err(|error| error.to_string())?;
    *shared = Some((path, environment.clone()));
    Ok(environment)
}
