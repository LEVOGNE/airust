use airust::execution::{CancellationToken, WorkControl};
use serde_json::json;
use std::{path::Path, time::Duration};

mod reference_support;
use reference_support::{Result, Workspace};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("build") if args.len() == 4 => {
            let workspace = Workspace::import(Path::new(&args[1]), &args[3])?;
            workspace.save(Path::new(&args[2]))?;
            println!(
                "{}",
                json!({"collection": workspace.collection, "documents": workspace.snapshots.len(), "passages": workspace.index.len(), "workspace": args[2]})
            );
        }
        Some("query") if args.len() == 3 => {
            let workspace = Workspace::load(Path::new(&args[1]))?;
            let control =
                WorkControl::new(CancellationToken::new()).with_timeout(Duration::from_secs(2))?;
            println!("{}", workspace.query(&args[2], control)?);
        }
        Some("serve") if args.len() == 2 => {
            let server = Workspace::load(Path::new(&args[1]))?.into_server()?;
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()?
                .block_on(server.serve(tokio::io::stdin(), tokio::io::stdout()))?;
        }
        Some("--help" | "help") | None => {
            println!("reference_workspace build INPUT_DIRECTORY NEW_WORKSPACE COLLECTION\nreference_workspace query WORKSPACE QUESTION\nreference_workspace serve WORKSPACE");
        }
        _ => return Err("invalid arguments; use reference_workspace --help".into()),
    }
    Ok(())
}
