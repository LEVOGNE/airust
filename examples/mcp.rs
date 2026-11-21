use airust::{
    mcp::{McpLimits, McpServer},
    KnowledgeEntry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = McpServer::from_entries(
        [
            KnowledgeEntry::new("ownership", "Rust ownership manages memory safely."),
            KnowledgeEntry::new("borrowing", "Rust borrowing uses references."),
        ],
        McpLimits::default(),
    )?;
    println!("{}", serde_json::to_string_pretty(&server.stats())?);
    for tool in server.tool_definitions() {
        println!("{}: {}", tool.name, tool.description.unwrap_or_default());
    }
    Ok(())
}
