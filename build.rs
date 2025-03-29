use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    fs::copy(
        "knowledge/train.json",
        Path::new(&out_dir).join("train.json"),
    )
    .unwrap();
}
