use airust::mcp::{McpLimits, McpServer};
use serde_json::json;
#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: load-heap KNOWLEDGE.json PROFILE.json EXPECTED_ENTRIES".into());
    }
    let expected: usize = args[3].parse()?;
    assert!((1..=1_000_000).contains(&expected));
    let source_bytes = std::fs::metadata(&args[1])?.len();
    let profiler = dhat::Profiler::builder().file_name(&args[2]).build();
    let server = McpServer::load_knowledge(&args[1], McpLimits::default())?;
    let entries = server.stats().entries;
    assert_eq!(entries, expected);
    assert_eq!(server.stats().in_flight, 0);
    let loaded = dhat::HeapStats::get();
    drop(server);
    let dropped = dhat::HeapStats::get();
    drop(profiler);
    println!(
        "{}",
        json!({"version":airust::VERSION,"entries":entries,"source_bytes":source_bytes,"max_live_bytes_before_server_drop":loaded.max_bytes,"max_live_blocks_before_server_drop":loaded.max_blocks,"live_bytes_after_load":loaded.curr_bytes,"live_blocks_after_load":loaded.curr_blocks,"allocated_bytes":loaded.total_bytes,"allocation_blocks":loaded.total_blocks,"live_bytes_after_server_drop":dropped.curr_bytes,"live_blocks_after_server_drop":dropped.curr_blocks,"scope":"Separate DHAT-instrumented startup load: raw read, owned JSON parse, index and schema construction included. Peak/live allocation counts exclude profiler metadata and are not process RSS. No duration used as latency evidence; no searches/norm warming. Small process-global SDK caches may remain after dropping the server."})
    );
    Ok(())
}
