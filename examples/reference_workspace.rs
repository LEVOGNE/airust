use airust::execution::{CancellationToken, WorkControl};
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};

mod pilot_support;
#[cfg(feature = "pilot")]
mod reference_profiles;
mod reference_support;
use reference_support::{Result, Workspace};

fn control() -> Result<WorkControl> {
    Ok(WorkControl::new(CancellationToken::new()).with_timeout(Duration::from_secs(2))?)
}
fn query(workspace: &Workspace, question: &str, profile: Option<&str>) -> Result<Value> {
    if let Some(path) = profile {
        #[cfg(feature = "pilot")]
        {
            let runtime = reference_profiles::Profile::load(workspace, Path::new(path))?;
            return workspace.query_using(question, control()?, |q, options| {
                Ok(runtime.query(&workspace.index, q, airust::SearchAlgorithm::Bm25, options)?)
            });
        }
        #[cfg(not(feature = "pilot"))]
        {
            let _ = path;
            return Err("compile with --features pilot for explicit model profiles".into());
        }
    }
    workspace.query(question, control()?)
}
fn profile_argument(args: &[String], offset: usize) -> Result<Option<&str>> {
    if args.len() == offset {
        Ok(None)
    } else if args.len() == offset + 2 && args[offset] == "--profile" {
        Ok(Some(&args[offset + 1]))
    } else {
        Err("expected an optional --profile CONFIG argument".into())
    }
}
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
        Some("query") if args.len() >= 3 => {
            let profile = profile_argument(&args, 3)?;
            let workspace = Workspace::load(Path::new(&args[1]))?;
            println!("{}", query(&workspace, &args[2], profile)?);
        }
        Some("serve") if args.len() >= 2 => {
            let profile = profile_argument(&args, 2)?;
            let workspace = Workspace::load(Path::new(&args[1]))?;
            let server = if let Some(path) = profile {
                #[cfg(feature = "pilot")]
                {
                    let backend = reference_profiles::Profile::load(&workspace, Path::new(path))?;
                    workspace.into_server()?.with_search_backend(backend)
                }
                #[cfg(not(feature = "pilot"))]
                {
                    let _ = path;
                    return Err("compile with --features pilot for explicit model profiles".into());
                }
            } else {
                workspace.into_server()?
            };
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()?
                .block_on(server.serve(tokio::io::stdin(), tokio::io::stdout()))?;
        }
        Some("profile-build") if args.len() == 4 => {
            #[cfg(feature = "pilot")]
            {
                let workspace = Workspace::load(Path::new(&args[1]))?;
                println!(
                    "{}",
                    reference_profiles::prepare(
                        &workspace,
                        Path::new(&args[2]),
                        Path::new(&args[3])
                    )?
                );
            }
            #[cfg(not(feature = "pilot"))]
            {
                return Err("compile with --features pilot to prepare model vectors".into());
            }
        }
        Some("pilot") => match args.get(1).map(String::as_str) {
            Some("init") if args.len() == 5 => println!(
                "{}",
                pilot_support::init(Path::new(&args[2]), Path::new(&args[3]), &args[4])?
            ),
            Some("capture") if args.len() >= 5 => {
                let profile = profile_argument(&args, 5)?;
                let output = Path::new(&args[4]);
                if output.exists() {
                    return Err("case destination already exists".into());
                }
                let request = pilot_support::read_request(Path::new(&args[3]))?;
                let workspace = Workspace::load(&Path::new(&args[2]).join("workspace"))?;
                let start = Instant::now();
                let response = query(&workspace, &request.query, profile)?;
                let case = pilot_support::PilotCase::capture(
                    &workspace,
                    request,
                    &response,
                    start.elapsed().as_secs_f64() * 1000.0,
                )?;
                case.save(output)?;
                println!(
                    "{}",
                    json!({"case_id":case.request.case_id,"workspace":case.workspace,"review_status":"unreviewed","origin":case.request.origin,"case":args[4]})
                );
            }
            Some("review") if args.len() == 6 => {
                let workspace = Workspace::load(&Path::new(&args[2]).join("workspace"))?;
                let case = pilot_support::PilotCase::load(&workspace, Path::new(&args[3]))?
                    .reviewed(&workspace, Path::new(&args[4]))?;
                case.save(Path::new(&args[5]))?;
                println!(
                    "{}",
                    json!({"case_id":case.request.case_id,"review_status":"caller_declared","case":args[5]})
                );
            }
            Some("export") if args.len() == 5 => {
                let workspace = Workspace::load(&Path::new(&args[2]).join("workspace"))?;
                println!(
                    "{}",
                    pilot_support::export(&workspace, Path::new(&args[3]), Path::new(&args[4]))?
                );
            }
            Some("check") if args.len() == 4 => {
                let workspace = Workspace::load(&Path::new(&args[2]).join("workspace"))?;
                let case = pilot_support::PilotCase::load(&workspace, Path::new(&args[3]))?;
                println!(
                    "{}",
                    json!({"case_id":case.request.case_id,"source_ranges":"verified","review_identity":"caller declaration, not authenticated"})
                );
            }
            _ => return Err("invalid pilot arguments; use --help".into()),
        },
        Some("--help" | "help") | None => {
            println!("reference_workspace build INPUT_DIRECTORY NEW_WORKSPACE COLLECTION\nreference_workspace query WORKSPACE QUESTION [--profile CONFIG]\nreference_workspace serve WORKSPACE [--profile CONFIG]\nreference_workspace profile-build WORKSPACE CONFIG NEW_VECTORS\nreference_workspace pilot init INPUT_DIRECTORY NEW_PILOT COLLECTION\nreference_workspace pilot capture PILOT REQUEST_JSON NEW_CASE_JSON [--profile CONFIG]\nreference_workspace pilot review PILOT CASE_JSON REVIEW_JSON NEW_CASE_JSON\nreference_workspace pilot check PILOT CASE_JSON\nreference_workspace pilot export PILOT SELECTED_CASE_DIRECTORY NEW_EXPORT");
        }
        _ => return Err("invalid arguments; use reference_workspace --help".into()),
    }
    Ok(())
}
