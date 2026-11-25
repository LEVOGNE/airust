#![forbid(unsafe_code)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use airust::reranking::process::{run_worker, WORKER_ARGUMENT};
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 || args[0] != WORKER_ARGUMENT {
        return Err("internal trusted local-model worker; use ProcessReranker to launch it".into());
    }
    run_worker()?;
    Ok(())
}
