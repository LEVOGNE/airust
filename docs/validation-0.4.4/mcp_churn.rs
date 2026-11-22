use airust::{
    mcp::{McpLimits, McpServer},
    KnowledgeEntry,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::{error::Error, fs, path::Path, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

async fn connection(server: McpServer) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (client, transport) = tokio::io::duplex(65536);
    let (reader, mut writer) = tokio::io::split(client);
    let (r, w) = tokio::io::split(transport);
    let task = tokio::spawn(server.serve(r, w));
    let mut reader = BufReader::new(reader);
    writer.write_all(b"{\"jsonrpc\":\"2.0\",\"id\":0,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"churn-probe\",\"version\":\"1\"}}}\n").await?;
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    assert_eq!(serde_json::from_str::<Value>(&line)?["id"], 0);
    writer
        .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
        .await?;
    for (id, name, arguments) in [
        (
            1,
            "airust_search",
            json!({"query":"common marker","limit":3}),
        ),
        (2, "airust_status", json!({})),
    ] {
        let mut bytes = serde_json::to_vec(
            &json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":arguments,"_meta":{"progressToken":format!("token-{id}")}}}),
        )?;
        bytes.push(b'\n');
        writer.write_all(&bytes).await?;
        let mut previous = None;
        loop {
            line.clear();
            assert!(reader.read_line(&mut line).await? > 0);
            let reply: Value = serde_json::from_str(&line)?;
            if reply["method"] == "notifications/progress" {
                assert_eq!(reply["params"]["progressToken"], format!("token-{id}"));
                let progress = reply["params"]["progress"].as_f64().unwrap();
                assert!(previous.is_none_or(|old| progress > old));
                previous = Some(progress);
            } else {
                assert_eq!(reply["id"], id);
                assert!(
                    !reply["result"]["isError"].as_bool().unwrap_or(false),
                    "unexpected {name} error: {}",
                    reply["result"]["structuredContent"]["error"]
                );
                assert!(reply["result"].get("structuredContent").is_some());
                break;
            }
        }
    }
    writer.shutdown().await?;
    task.await??;
    Ok(())
}
async fn batch(server: &McpServer, parallel: usize) {
    let mut work = Vec::new();
    for _ in 0..parallel {
        let server = server.clone();
        work.push(tokio::spawn(async move {
            tokio::time::timeout(Duration::from_secs(10), connection(server))
                .await
                .unwrap()
                .unwrap();
        }));
    }
    for task in work {
        task.await.unwrap();
    }
    assert_eq!(server.stats().in_flight, 0);
}
#[derive(Clone, Copy, Serialize)]
struct Reading {
    live_bytes: usize,
    live_blocks: usize,
    peak_bytes: usize,
    total_bytes: u64,
    total_blocks: u64,
}
#[derive(Serialize)]
struct Sample {
    closed_connections: usize,
    heap: Reading,
}
fn stats() -> Reading {
    let s = dhat::HeapStats::get();
    Reading {
        live_bytes: s.curr_bytes,
        live_blocks: s.curr_blocks,
        peak_bytes: s.max_bytes,
        total_bytes: s.total_bytes,
        total_blocks: s.total_blocks,
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(1..=2).contains(&args.len()) {
        return Err("usage: mcp-churn NEW_OUTPUT_DIR [PARALLEL=4]".into());
    }
    let parallel: usize = args.get(1).map(|s| s.parse()).transpose()?.unwrap_or(4);
    if ![1, 4].contains(&parallel) {
        return Err("parallel must be 1 or 4".into());
    }
    let output = Path::new(&args[0]);
    fs::create_dir(output)?;
    let server = McpServer::from_entries(
        (0..1000)
            .map(|n| KnowledgeEntry::new(n.to_string(), format!("common marker topic{}", n % 17))),
        McpLimits::default(),
    )?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    runtime.block_on(async {
        for _ in 0..10 {
            batch(&server, parallel).await;
        }
    });
    let mut samples = Vec::with_capacity(10);
    let profiler = dhat::Profiler::builder()
        .file_name(output.join("private-churn.dhat.json"))
        .build();
    runtime.block_on(async {
        for _ in 0..10 {
            batch(&server, parallel).await;
        }
    });
    let baseline = stats();

    for step in 0..(1000 / parallel) {
        runtime.block_on(batch(&server, parallel));
        if ((step + 1) * parallel) % 100 == 0 {
            let sample = stats();
            eprintln!(
                "closed_connections={} tracked_live_bytes={}",
                (step + 1) * parallel,
                sample.live_bytes
            );
            samples.push(Sample {
                closed_connections: (step + 1) * parallel,
                heap: sample,
            });
        }
    }
    let final_heap = stats();
    assert!(
        final_heap.live_bytes <= baseline.live_bytes + 256 * 1024,
        "unexpected retained connection allocation growth"
    );
    drop(server);
    drop(runtime);
    let released = stats();
    drop(profiler);
    let result = json!({"version":airust::VERSION,"indexed_entries":1000,"warmup_connections":20*parallel,"measured_connections":1000,"tools_per_connection":2,"concurrent_connections":parallel,"all_tools_complete":true,"progress_tokens_and_sequences_checked":true,"all_connection_workers_released":true,"baseline":baseline,"after_connections":final_heap,"after_server_and_runtime_drop":released,"samples":samples,"scope":"Allocator-instrumented same-process cloned-server churn. Index/runtime/cached definitions are preloaded outside profiling. Numeric telemetry storage is preallocated before profiling; JSON is built afterward. Live counts may include tracked runtime caches.256KiB growth guard is not a proof of no leaks. No latency measurements or OS-RSS guarantee."});
    fs::write(
        output.join("summary.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{}", result);
    Ok(())
}
