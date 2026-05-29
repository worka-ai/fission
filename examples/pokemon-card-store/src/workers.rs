use serde_json::{json, Value};

pub fn catalog_filters_boot(input: &str) -> String {
    let message: Value = serde_json::from_str(input).unwrap_or_else(|_| json!({ "type": "boot" }));
    json!({
        "messages": [
            {
                "type": "dom_batch",
                "sequence": message.get("sequence").and_then(Value::as_u64).unwrap_or(1),
                "transaction_id": "catalog-filters",
                "ops": [
                    { "op": "set_text_by_semantics", "semantics": "worker-status:catalog-filters", "text": "Worker bridge ready" },
                    { "op": "set_text_by_semantics", "semantics": "worker-filter-summary", "text": "Browser worker loaded the route manifest and can now enhance catalogue filters." }
                ]
            }
        ],
        "bindings": []
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_boot_emits_status_updates() {
        let output = catalog_filters_boot(r#"{"type":"boot"}"#);
        assert!(output.contains("Worker bridge ready"));
        assert!(output.contains("worker-filter-summary"));
    }
}
