//! Finite memory probe for retained input errors; creates only owned fixtures.
use airust::{
    documents::{DocumentError, DocumentLimits, DocumentSnapshot},
    mcp::{McpError, McpLimits, McpServer},
    AgentError, Engine, IndexError, KnowledgeBase,
};
use std::{error::Error, hint::black_box};

fn rejected<T, E: Error + 'static>(result: Result<T, E>) -> Box<dyn Error> {
    match result {
        Ok(_) => panic!("malformed input accepted"),
        Err(error) => Box::new(error),
    }
}
fn data_error<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a serde_json::Error> {
    if let Some(McpError::Json(error)) = error.downcast_ref::<McpError>() {
        return Some(error);
    }
    if let Some(AgentError::SerializationError(error)) = error.downcast_ref::<AgentError>() {
        return Some(error);
    }
    if let Some(IndexError::Json(error)) = error.downcast_ref::<IndexError>() {
        return Some(error);
    }
    if let Some(DocumentError::Json(error)) = error.downcast_ref::<DocumentError>() {
        return Some(error);
    }
    None
}
fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "utf8".into());
    if ![
        "utf8",
        "mcp-json",
        "training-json",
        "engine-json",
        "document-json",
    ]
    .contains(&mode.as_str())
    {
        return Err(
            "expected retained_errors [utf8|mcp-json|training-json|engine-json|document-json]"
                .into(),
        );
    }
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("invalid.json");
    let bytes = 2 * 1024 * 1024;
    let input = if mode == "utf8" {
        let mut input = vec![0; bytes];
        input[0] = 0xFF;
        input
    } else {
        serde_json::to_vec(&"x".repeat(bytes))?
    };
    std::fs::write(&path, &input)?;
    let limits = McpLimits {
        max_file_bytes: 4 * 1024 * 1024,
        ..Default::default()
    };
    let mut retained = Vec::new();
    for _ in 0..128 {
        let error = match mode.as_str() {
            "utf8" | "mcp-json" => rejected(McpServer::load_knowledge(&path, limits.clone())),
            "training-json" => rejected(KnowledgeBase::load(path.clone())),
            "engine-json" => rejected(Engine::from_json(std::str::from_utf8(&input)?)),
            "document-json" => rejected(DocumentSnapshot::from_json(
                std::str::from_utf8(&input)?,
                &DocumentLimits::default(),
            )),
            _ => unreachable!(),
        };
        if mode == "utf8" {
            assert!(
                matches!(error.downcast_ref::<McpError>(), Some(McpError::Io(error)) if error.kind()==std::io::ErrorKind::InvalidData)
            );
        } else {
            let json = data_error(error.as_ref()).ok_or("JSON error category changed")?;
            assert!(json.is_data());
            assert_eq!(json.line(), 1);
            assert_eq!(json.column(), input.len());
        }
        retained.push(error);
    }
    assert_eq!(std::fs::read(&path)?, input);
    black_box(&retained);
    println!(
        "{}",
        serde_json::json!({"mode":mode,"retained_errors":retained.len(),"input_bytes":input.len(),"error_message_bytes":retained[0].to_string().len(),"original_unchanged":true})
    );
    Ok(())
}
