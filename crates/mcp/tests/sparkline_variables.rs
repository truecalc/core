//! A sparkline emitted by the MCP `evaluate` tool must be usable as a variable
//! on the way back in.
//!
//! `value_to_json` emits `{ "type": "sparkline", "value": {...} }`. Without the
//! matching decode in `parse_variables` that object matches no branch and the
//! binding is silently dropped to `empty`, so `TYPE(x)` answers 1 instead of
//! 128 and `ISBLANK(x)` answers TRUE — a wrong answer, not an error.

use serde_json::{json, Value as JsonValue};
use std::io::Write;

/// Run one `evaluate` call against the binary and return its inner JSON result.
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

/// The emitted form of a sparkline, taken from the server itself rather than
/// hand-written, so the test breaks if either direction drifts.
fn emitted_sparkline() -> JsonValue {
    let value = evaluate("=SPARKLINE({1,2,3},{\"color\",\"red\"})", json!({}));
    assert_eq!(value["type"], json!("sparkline"), "emitted {value}");
    value
}

#[test]
fn a_sparkline_variable_is_still_a_sparkline() {
    let result = evaluate("=TYPE(x)", json!({ "x": emitted_sparkline() }));
    assert_eq!(
        result["value"],
        json!(128.0),
        "a sparkline variable must keep its own value kind, got {result}"
    );
}

#[test]
fn a_sparkline_variable_is_not_blank() {
    let result = evaluate("=ISBLANK(x)", json!({ "x": emitted_sparkline() }));
    assert_eq!(result["value"], json!(false), "got {result}");
}

#[test]
fn a_malformed_sparkline_variable_is_dropped_rather_than_decoded() {
    // The decoder accepts exactly what this server can emit and nothing wider:
    // an unknown charttype, a `data` array shorter than two points (the
    // evaluator answers `#N/A` for such a call, so it cannot emit one), and
    // `charttype` left in the option list (it is always lifted into its own
    // field). A rejected payload drops the binding — `TYPE(x)` then reports an
    // unbound name's kind, never 128.
    let point = json!({ "value": 1.0, "type": "number" });
    let unbound = evaluate("=TYPE(x)", json!({}));
    for bad in [
        // Two valid points, so the unknown charttype is what rejects this —
        // with an empty `data` the length guard would fire first and this case
        // would pass even with the charttype check deleted.
        json!({ "type": "sparkline", "value": {
            "charttype": "bogus",
            "data": [point.clone(), point.clone()],
            "options": [] } }),
        json!({ "type": "sparkline", "value": {
            "charttype": "line", "data": [], "options": [] } }),
        json!({ "type": "sparkline", "value": {
            "charttype": "line", "data": [point.clone()], "options": [] } }),
        json!({ "type": "sparkline", "value": {
            "charttype": "line",
            "data": [point.clone(), point.clone()],
            "options": [["charttype", { "value": "bar", "type": "text" }]] } }),
        // An option that is not a [key, value] pair. Without the length guard
        // this indexes out of bounds — a panic in a server decoding
        // caller-supplied JSON, not a rejection.
        json!({ "type": "sparkline", "value": {
            "charttype": "line",
            "data": [point.clone(), point.clone()],
            "options": [["color"]] } }),
        json!({ "type": "sparkline", "value": {
            "charttype": "line",
            "data": [point.clone(), point.clone()],
            "options": [["color", { "value": "red", "type": "text" }, "extra"]] } }),
        // A data point whose payload does not match its own tag.
        json!({ "type": "sparkline", "value": {
            "charttype": "line",
            "data": [point.clone(), { "value": "not a number", "type": "number" }],
            "options": [] } }),
        json!({ "type": "sparkline", "value": {
            "charttype": "line",
            "data": [point.clone(), { "value": 1.0, "type": "unknown" }],
            "options": [] } }),
    ] {
        let result = evaluate("=TYPE(x)", json!({ "x": bad.clone() }));
        assert_ne!(result["value"], json!(128.0), "decoded {bad}");
        assert_eq!(result, unbound, "should have been dropped: {bad}");
    }
}

#[test]
fn a_sparkline_variable_round_trips_its_spec() {
    // COUNTUNIQUE is the only surface that can see the spec, so it is what
    // proves the payload survived rather than being rebuilt as some default.
    let spark = emitted_sparkline();
    let same = evaluate(
        "=COUNTUNIQUE(x,SPARKLINE({1,2,3},{\"color\",\"red\"}))",
        json!({ "x": spark.clone() }),
    );
    assert_eq!(same["value"], json!(1.0), "got {same}");

    let different = evaluate(
        "=COUNTUNIQUE(x,SPARKLINE({9,9,9}))",
        json!({ "x": spark }),
    );
    assert_eq!(different["value"], json!(2.0), "got {different}");
}
