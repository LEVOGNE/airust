use serde_json::json;
use std::{error::Error, time::Instant};
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: pdf_parser_profile INPUT.pdf")?;
    let start = Instant::now();
    let document = pdf_oxide::PdfDocument::open(path)?;
    let open_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let count = document.page_count()?;
    let page_count_ms = start.elapsed().as_secs_f64() * 1000.0;
    let mut pages = Vec::new();
    for page in 0..count {
        let start = Instant::now();
        let text = document.extract_text(page)?;
        pages.push(json!({"page":page+1,"bytes":text.len(),"extract_ms":start.elapsed().as_secs_f64()*1000.0}));
    }
    println!(
        "{}",
        json!({"version":airust::VERSION,"open_ms":open_ms,"page_count_ms":page_count_ms,"pages":pages,"limits":"Direct dependency extraction timings; no AIRust cleaning, chunking or search. Not a parser security audit."})
    );
    Ok(())
}
