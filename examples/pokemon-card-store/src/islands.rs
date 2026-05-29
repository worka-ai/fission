use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};

static CART_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn cart_drawer_boot(input: &str) -> String {
    let message: Value = serde_json::from_str(input).unwrap_or_else(|_| json!({ "type": "boot" }));
    let is_event = message
        .get("type")
        .and_then(Value::as_str)
        .is_some_and(|kind| kind == "event");
    let action = message
        .get("binding")
        .and_then(|binding| binding.get("message"))
        .and_then(|message| message.get("action"))
        .and_then(Value::as_str);

    if !is_event {
        CART_COUNT.store(0, Ordering::Relaxed);
    }

    let count = if is_event && action == Some("add") {
        CART_COUNT.fetch_add(1, Ordering::Relaxed) + 1
    } else {
        CART_COUNT.load(Ordering::Relaxed)
    };
    let item_word = if count == 1 { "item" } else { "items" };
    let status = if is_event {
        format!("Island handled {count} client event(s)")
    } else {
        "Island bridge ready".to_string()
    };

    json!({
        "messages": [
            {
                "type": "dom_batch",
                "sequence": message.get("sequence").and_then(Value::as_u64).unwrap_or(1),
                "transaction_id": "cart-drawer",
                "ops": [
                    { "op": "set_text_by_semantics", "semantics": "island-status:cart-drawer", "text": status },
                    { "op": "set_text_by_semantics", "semantics": "island-cart-count", "text": format!("{count} {item_word} in the browser island cart") },
                    { "op": "set_text_by_semantics", "semantics": "island-last-event", "text": if is_event { "Updated without a full page request" } else { "Ready for client-side cart edits" } },
                    { "op": "set_attr_by_semantics", "semantics": "island-action:add-card", "name": "role", "value": "button" },
                    { "op": "set_attr_by_semantics", "semantics": "island-action:add-card", "name": "tabindex", "value": "0" },
                    { "op": "add_class_by_semantics", "semantics": "island-action:add-card", "class": "fission-browser-action" }
                ]
            }
        ],
        "bindings": [
            {
                "semantics": "island-action:add-card",
                "event": "click",
                "message": { "action": "add" }
            }
        ]
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cart_island_boot_and_click_emit_dom_updates() {
        let boot = cart_drawer_boot(r#"{"type":"boot"}"#);
        assert!(boot.contains("Island bridge ready"));
        assert!(boot.contains("island-action:add-card"));

        let click = cart_drawer_boot(
            r#"{"type":"event","sequence":2,"binding":{"message":{"action":"add"}}}"#,
        );
        assert!(click.contains("1 item in the browser island cart"));
        assert!(click.contains("Updated without a full page request"));
    }
}
