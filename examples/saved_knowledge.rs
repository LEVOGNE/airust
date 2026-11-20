use airust::{
    retrieval::{LexicalSource, RetrievalOptions, Retriever},
    Engine,
};
use std::{error::Error, fs};
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: saved_knowledge INDEX.json QUERY; load a trusted local AIRust export".into(),
        );
    }
    let engine = Engine::from_json(&fs::read_to_string(&args[0])?)?;
    let response =
        Retriever::new(LexicalSource::new(&engine)).query(&args[1], &RetrievalOptions::new(10))?;
    println!("{}", response.to_json());
    Ok(())
}
