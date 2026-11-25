#![forbid(unsafe_code)]
//! Static token/padding analysis only; never executes a model or measures inference.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    io::{Read, Write},
    path::Path,
};
use tokenizers::{AddedToken, PaddingParams, PaddingStrategy, Tokenizer, TruncationParams};
type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Prefix {
    id: String,
    query: String,
    candidates: Vec<Entry>,
}
#[derive(Deserialize)]
struct Entry {
    id: String,
    text: String,
}
#[derive(Deserialize)]
struct Asset {
    bytes: u64,
    sha256: String,
}
#[derive(Deserialize)]
struct Manifest {
    revision: String,
    files: BTreeMap<String, Asset>,
}
fn bounded(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    let metadata = path.metadata()?;
    if !metadata.is_file() || metadata.len() > maximum {
        return Err("probe input is not a bounded regular file".into());
    }
    let file = fs::File::open(path)?;
    let opened = file.metadata()?;
    if !opened.is_file() || opened.len() > maximum || opened.len() != metadata.len() {
        return Err("probe input changed before opening".into());
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(usize::try_from(opened.len() + 1)?)?;
    file.take(opened.len() + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != metadata.len() {
        return Err("probe input changed length".into());
    }
    Ok(bytes)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn put(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 4 {
        return Err("expected padding-probe PREFIXES_JSON MODEL_DIRECTORY MANIFEST_JSON NEW_OUTPUT_DIRECTORY".into());
    }
    let input = bounded(Path::new(&args[0]), 64 * 1024 * 1024)?;
    let prefixes: Vec<Prefix> = serde_json::from_slice(&input)?;
    assert_eq!(prefixes.len(), 150);
    assert!(prefixes.iter().all(|p| {
        !p.id.is_empty()
            && p.id.len() <= 64
            && !p.query.trim().is_empty()
            && p.query.len() <= 4096
            && p.candidates.len() == 20
            && p.candidates.iter().all(|entry| {
                !entry.id.is_empty()
                    && entry.id.len() <= 256
                    && !entry.text.trim().is_empty()
                    && entry.text.len() <= 1800
            })
    }));
    let model = Path::new(&args[1]);
    let manifest_bytes = bounded(Path::new(&args[2]), 64 * 1024)?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
    let mut assets = BTreeMap::new();
    for name in [
        "tokenizer.json",
        "config.json",
        "tokenizer_config.json",
        "special_tokens_map.json",
    ] {
        let expected = &manifest.files[name];
        let bytes = bounded(&model.join(name), 64 * 1024 * 1024)?;
        assert_eq!(bytes.len() as u64, expected.bytes);
        assert_eq!(digest(&bytes), expected.sha256);
        assets.insert(name, bytes);
    }
    let config: serde_json::Value = serde_json::from_slice(&assets["config.json"])?;
    let tokenizer_config: serde_json::Value =
        serde_json::from_slice(&assets["tokenizer_config.json"])?;
    let special: serde_json::Value = serde_json::from_slice(&assets["special_tokens_map.json"])?;
    assert_eq!(tokenizer_config["model_max_length"].as_u64(), Some(512));
    let pad = tokenizer_config["pad_token"]
        .as_str()
        .ok_or("unexpected pad-token representation")?;
    let pad_id = config["pad_token_id"]
        .as_u64()
        .ok_or("missing pad-token ID")? as u32;
    let mut tokenizer = Tokenizer::from_bytes(&assets["tokenizer.json"])?;
    // Match FastEmbed 7.7.1's pinned special-token setup, including the
    // explicitly configured whitespace/normalization flags for object tokens.
    for value in special
        .as_object()
        .ok_or("unexpected special-token map")?
        .values()
    {
        let added = if let Some(text) = value.as_str() {
            if tokenizer.get_added_vocabulary().is_special_token(text) {
                continue;
            }
            AddedToken {
                content: text.into(),
                special: true,
                ..Default::default()
            }
        } else {
            AddedToken {
                content: value["content"]
                    .as_str()
                    .ok_or("special token content")?
                    .into(),
                single_word: value["single_word"]
                    .as_bool()
                    .ok_or("special token single_word")?,
                lstrip: value["lstrip"].as_bool().ok_or("special token lstrip")?,
                rstrip: value["rstrip"].as_bool().ok_or("special token rstrip")?,
                normalized: value["normalized"]
                    .as_bool()
                    .ok_or("special token normalized")?,
                special: true,
            }
        };
        tokenizer.add_special_tokens([added])?;
    }
    tokenizer.with_padding(Some(PaddingParams {
        strategy: PaddingStrategy::BatchLongest,
        pad_token: pad.into(),
        pad_id,
        ..Default::default()
    }));
    tokenizer.with_truncation(Some(TruncationParams {
        max_length: 512,
        ..Default::default()
    }))?;
    let mut full = tokenizer.clone();
    full.with_truncation(None)?;
    let output = Path::new(&args[3]);
    fs::create_dir(output)?;
    let mut pairs = Vec::new();
    let mut batches = Vec::new();
    for case in &prefixes {
        let mut singles = Vec::new();
        for (position, entry) in case.candidates.iter().enumerate() {
            let single = tokenizer.encode((case.query.as_str(), entry.text.as_str()), true)?;
            let untruncated = full.encode((case.query.as_str(), entry.text.as_str()), true)?;
            let actual: usize = single
                .get_attention_mask()
                .iter()
                .map(|v| *v as usize)
                .sum();
            assert_eq!(actual, single.len());
            assert!(actual <= 512);
            let mut fingerprint = Sha256::new();
            for id in single.get_ids() {
                fingerprint.update(id.to_le_bytes());
            }
            pairs.push(serde_json::json!({"id":case.id,"document_id":entry.id,"position":position,"text_sha256":digest(entry.text.as_bytes()),"untruncated_tokens":untruncated.len(),"tokens":actual,"truncated":untruncated.len()>actual,"token_ids_sha256":format!("{:x}",fingerprint.finalize())}));
            singles.push(single);
        }
        for batch_size in [1, 4, 8, 16] {
            let mut cells = 0_u64;
            let mut attention = 0_u64;
            let mut actual = 0_u64;
            let mut actual_attention = 0_u64;
            for (offset, chunk) in case.candidates.chunks(batch_size).enumerate() {
                let encoded = tokenizer.encode_batch(
                    chunk
                        .iter()
                        .map(|e| (case.query.as_str(), e.text.as_str()))
                        .collect(),
                    true,
                )?;
                for (i, encoding) in encoded.iter().enumerate() {
                    let single = &singles[offset * batch_size + i];
                    let length = single.len();
                    assert_eq!(&encoding.get_ids()[..length], single.get_ids());
                    assert_eq!(
                        &encoding.get_attention_mask()[..length],
                        single.get_attention_mask()
                    );
                    assert!(encoding.get_attention_mask()[length..]
                        .iter()
                        .all(|v| *v == 0));
                    cells += encoding.len() as u64;
                    attention += (encoding.len() as u64).pow(2);
                    actual += length as u64;
                    actual_attention += (length as u64).pow(2);
                }
            }
            assert!(cells >= actual && attention >= actual_attention);
            batches.push(serde_json::json!({"id":case.id,"batch_size":batch_size,"pairs":20,"actual_token_cells":actual,"padded_token_cells":cells,"actual_attention_proxy":actual_attention,"padded_attention_proxy":attention}));
        }
    }
    put(&output.join("pairs.json"), &pairs)?;
    put(&output.join("batches.json"), &batches)?;
    put(
        &output.join("report.json"),
        &serde_json::json!({"prefixes_sha256":digest(&input),"manifest_sha256":digest(&manifest_bytes),"model_revision":manifest.revision,"tokenizer_sha256":digest(&assets["tokenizer.json"]),"queries":prefixes.len(),"pairs":pairs.len(),"batch_observations":batches.len(),"all_nonpadding_ids_and_masks_match_single_pair":true,"maximum_pair_tokens":512,"truncated_pairs":pairs.iter().filter(|p|p["truncated"]==true).count(),"scope":"Static tokenizer structure for the150authored reference prefixes. No model executed; token-cell and squared-length proxies are not measured FLOPs, latency, energy or quality gains."}),
    )?;
    Ok(())
}
