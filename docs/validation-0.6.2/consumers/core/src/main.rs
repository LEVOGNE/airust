use airust::{Engine,KnowledgeEntry};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut index=Engine::new();
 index.upsert(KnowledgeEntry::new("ownership","Rust ownership protects memory."))?;
 index.upsert(KnowledgeEntry::new("other","Other material."))?;
 assert_eq!(index.search("Rust memory",5)[0].entry.id,"ownership");
 println!("core query passed");
 Ok(())
}
