use serde_json::{json, Value};
use std::{collections::BTreeMap, error::Error, fs};
fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let data: Value = serde_json::from_reader(fs::File::open(&args[0])?)?;
    let tools = data["tools"].as_array().ok_or("missing tools")?;
    assert_eq!(tools.len(), 5);
    let mut validators = BTreeMap::new();
    let mut counts = BTreeMap::new();
    for tool in tools {
        let name = tool["name"].as_str().ok_or("tool name")?.to_owned();
        let schema = &tool["outputSchema"];
        jsonschema::meta::validate(schema)
            .map_err(|e| format!("Invalid output schema at {}", e.instance_path()))?;
        validators.insert(name, jsonschema::validator_for(schema)?);
    }
    let results = data["results"].as_array().ok_or("results")?;
    assert!(!results.is_empty());
    for (number, row) in results.iter().enumerate() {
        let name = row["tool"].as_str().ok_or("result tool")?;
        let result = &row["result"];
        let structured = &result["structuredContent"];
        assert!(
            validators
                .get(name)
                .ok_or("unknown tool")?
                .is_valid(structured),
            "Schema mismatch at response {number} for {name}"
        );
        let text: Value =
            serde_json::from_str(result["content"][0]["text"].as_str().ok_or("text copy")?)?;
        assert!(
            text == *structured,
            "Text and structured data differ at response {number}"
        );
        assert_eq!(
            result["isError"].as_bool().unwrap_or(false),
            structured.get("error").is_some()
        );
        *counts.entry(name.to_owned()).or_insert(0usize) += 1;
    }
    let output = json!({"schema_draft":"2020-12","tool_schemas":5,"actual_results_validated":results.len(),"results_by_tool":counts,"all_text_copies_equal":true,"remote_reference_resolution":false});
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[1])?;
    serde_json::to_writer_pretty(file, &output)?;
    println!("{output}");
    Ok(())
}
