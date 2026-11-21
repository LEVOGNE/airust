use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, LazyLock},
};

static OUTPUTS: LazyLock<BTreeMap<&'static str, Arc<Map<String, Value>>>> = LazyLock::new(|| {
    let bundle: Value = serde_json::from_str(include_str!("../../docs/schemas/mcp-output-v1.json"))
        .expect("bundled MCP output schemas");
    [
        "airust_search",
        "airust_read_passage",
        "airust_find_matches",
        "airust_relate",
        "airust_status",
    ]
    .into_iter()
    .map(|name| {
        let mut needed = BTreeSet::new();
        let mut queue = vec!["Error".to_owned(), name.to_owned()];
        let mut definitions = Map::new();
        while let Some(key) = queue.pop() {
            if !needed.insert(key.clone()) {
                continue;
            }
            let value = bundle["$defs"][&key].clone();
            assert!(value.is_object(), "bundled MCP schema reference");
            references(&value, &mut queue);
            definitions.insert(key, value);
        }
        let output = json!({
            "$schema": bundle["$schema"],
            "$id": format!("urn:airust:mcp:{}:{name}", crate::VERSION),
            "type": "object",
            "oneOf": [
                {"$ref": "#/$defs/Error"},
                {"$ref": format!("#/$defs/{name}")}
            ],
            "$defs": definitions
        });
        (
            name,
            Arc::new(output.as_object().cloned().expect("MCP schema object")),
        )
    })
    .collect()
});
fn references(value: &Value, queue: &mut Vec<String>) {
    match value {
        Value::Object(values) => {
            if let Some(key) = values
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(|s| s.strip_prefix("#/$defs/"))
            {
                queue.push(key.to_owned());
            }
            for value in values.values() {
                references(value, queue);
            }
        }
        Value::Array(values) => {
            for value in values {
                references(value, queue);
            }
        }
        _ => {}
    }
}
pub(super) fn output(name: &str) -> Arc<Map<String, Value>> {
    OUTPUTS.get(name).expect("known MCP output schema").clone()
}
