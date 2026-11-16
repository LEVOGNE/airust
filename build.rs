use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rerun-if-changed=knowledge/train.json");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_EMBEDDED_KNOWLEDGE");
    let src = Path::new("knowledge/train.json");
    let dst = Path::new(&out_dir).join("train.json");

    let data = if env::var_os("CARGO_FEATURE_EMBEDDED_KNOWLEDGE").is_some() && src.exists() {
        fs::read(src).expect("Failed to read embedded knowledge/train.json")
    } else {
        b"[]".to_vec()
    };
    fs::write(dst, data).expect("Failed to write train.json to OUT_DIR");
}
