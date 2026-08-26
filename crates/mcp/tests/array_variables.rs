//! An array variable binding evaluates as an array.
//!
//! `value_to_json` already emits `{ "type": "array", "value": [...] }`, so
//! arrays were part of this server's vocabulary on the way out but had no
//! decode on the way in. A bare JSON array is what a caller writes, and
//! `Value::Array` is what the evaluator already ranges over, so the binding
//! needs no special handling past the decode.

use serde_json::{json, Value as JsonValue};
use std::io::Write;

fn evaluate(formula: &str, variables: JsonValue) -> JsonValue {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_truecalc-mcp"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start truecalc-mcp");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "evaluate",
            "arguments": { "formula": formula, "variables": variables }
        }
    });

    let stdin = child.stdin.as_mut().expect("stdin");
    writeln!(stdin, "{}", serde_json::to_string(&request).unwrap()).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let response: JsonValue = serde_json::from_str(stdout.trim()).expect("json");
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default();
    serde_json::from_str(text).expect("inner json")
}

/// The reproduction from the report, asserted on the answers themselves. Every
/// one of these previously came back as the answer for an unbound name: 0, 0,
/// 0, 0, 10, `#DIV/0!` and TRUE respectively.
#[test]
fn an_array_binding_ranges_like_an_array() {
    let a = json!({ "a": [1, 2, 3] });
    for (formula, expected) in [
        ("=SUM(a)", json!(6.0)),
        ("=COUNT(a)", json!(3.0)),
        ("=COUNTA(a)", json!(3.0)),
        ("=MAX(a)", json!(3.0)),
        ("=SUM(a, 10)", json!(16.0)),
        ("=AVERAGE(a)", json!(2.0)),
        ("=ISBLANK(a)", json!(false)),
        ("=INDEX(a, 2)", json!(2.0)),
    ] {
        let result = evaluate(formula, a.clone());
        assert_eq!(result["value"], expected, "{formula} gave {result}");
    }
}

/// The binding keeps its own kind rather than collapsing to its first element.
#[test]
fn an_array_binding_is_an_array_value() {
    let result = evaluate("=a", json!({ "a": [1, 2, 3] }));
    assert_eq!(result["type"], json!("array"), "got {result}");
    assert_eq!(
        result["value"],
        json!([
            { "value": 1.0, "type": "number" },
            { "value": 2.0, "type": "number" },
            { "value": 3.0, "type": "number" },
        ]),
        "got {result}"
    );
}

/// Elements decode by the same rules as a top-level binding, so an array is not
/// limited to numbers.
#[test]
fn array_elements_may_be_any_supported_scalar() {
    let vars = json!({ "a": ["x", true, 2] });
    assert_eq!(
        evaluate("=COUNTA(a)", vars.clone())["value"],
        json!(3.0),
        "text and boolean elements must survive the decode"
    );
    assert_eq!(
        evaluate("=CONCATENATE(INDEX(a, 1), \"!\")", vars)["value"],
        json!("x!"),
        "a text element must arrive as text"
    );
}

/// An element the server cannot decode still fails the call by name — adding
/// array support must not reopen the silent-drop hole inside an array.
#[test]
fn an_undecodable_element_is_still_reported() {
    let result = evaluate("=SUM(a)", json!({ "a": [1, null, 3] }));
    let err = result["error"]
        .as_str()
        .unwrap_or_else(|| panic!("expected an error, got a value: {result}"));
    assert!(
        err.contains("a"),
        "the error must name the binding, got {err:?}"
    );
    assert!(
        result.get("value").is_none(),
        "a rejected binding must not also yield a value: {result}"
    );
}

/// An empty array is a binding, not a missing one.
#[test]
fn an_empty_array_binding_is_accepted() {
    let result = evaluate("=COUNT(a)", json!({ "a": [] }));
    assert_eq!(result["value"], json!(0.0), "got {result}");
    assert!(result.get("error").is_none(), "got {result}");
}
