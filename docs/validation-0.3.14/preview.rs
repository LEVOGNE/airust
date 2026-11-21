use sha2::{Digest, Sha256};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let path = std::path::Path::new(&args[2]);
    let start = std::time::Instant::now();
    let pages = match args[1].as_str() {
        "0.3.13" => previous::PdfLoader::new().with_limits(previous::PdfImportLimits::default()).preview_pages_from_pdf(path, 1)?.into_iter().map(|p| (p.page_number, p.text)).collect::<Vec<_>>(),
        "0.3.14" => current::PdfLoader::new().with_limits(current::PdfImportLimits::default()).preview_pages_from_pdf(path, 1)?.into_iter().map(|p| (p.page_number, p.text)).collect::<Vec<_>>(),
        _ => return Err("unknown version".into()),
    };
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(pages.len(), 1);
    let data = serde_json::to_vec(&pages)?;
    println!("{}", serde_json::json!({"version":args[1],"requested_usable_pages":1,"returned_physical_pages":pages.iter().map(|p|p.0).collect::<Vec<_>>(),"result_sha256":format!("{:x}",Sha256::digest(&data)),"result_bytes":data.len(),"api_ms":elapsed}));
    Ok(())
}
