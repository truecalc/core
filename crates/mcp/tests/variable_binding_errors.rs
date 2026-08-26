//! A variable binding the server cannot decode must be reported, never dropped.
//!
//! A dropped binding leaves the name unbound, and an unbound name evaluates to
//! empty rather than raising. The caller then gets a plausible wrong number
//! instead of an error: with `{"a": [1,2,3]}` dropped, `SUM(a)` answered `0`
//! and `SUM(a, 10)` answered `10` — a number that looks entirely reasonable in
//! a formula mixing a variable with scalar terms, and is wrong. Arrays decode
//! now; every shape that still has no decode must be reported, not dropped.

use serde_json::{json, Value as JsonValue};
use std::io::Write;

/// Run one tool call against the binary and return its inner JSON result.
fn call(name: &str, arguments: JsonValue) -> JsonValue {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_truecalc-mcp"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start truecalc-mcp");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": { "name": name, "arguments": arguments }
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

fn evaluate(formula: &str, variables: JsonValue) -> JsonValue {
    call(
        "evaluate",
        json!({ "formula": formula, "variables": variables }),
    )
}

/// The error text for a binding the server rejected, asserting that it is an
/// error at all and that it names the binding it is about.
fn rejection(result: &JsonValue, name: &str) -> String {
    let err = result["error"]
        .as_str()
        .unwrap_or_else(|| panic!("expected an error, got a value: {result}"));
    assert!(
        err.contains(name),
        "the error must name the offending binding {name:?}, got {err:?}"
    );
    err.to_owned()
}

/// The bindings this server has no decode for. Each must be reported by name;
/// none may reach the evaluator as an unbound — and therefore empty — name.
fn undecodable() -> Vec<(&'static str, JsonValue)> {
    let point = json!({ "value": 1.0, "type": "number" });
    vec![
        ("null", json!(null)),
        ("plain object", json!({ "x": 1 })),
        // `{"type": "zoned"}` whose payload is not an RFC-9557 instant.
        (
            "malformed zoned",
            json!({ "type": "zoned", "value": "not an instant" }),
        ),
        ("zoned with no value", json!({ "type": "zoned" })),
        // `{"type": "sparkline"}` outside what this server can emit: an unknown
        // charttype, and a `data` array shorter than the two points the
        // evaluator requires.
        (
            "sparkline with unknown charttype",
            json!({ "type": "sparkline", "value": {
                "charttype": "bogus",
                "data": [point.clone(), point.clone()],
                "options": [] } }),
        ),
        (
            "sparkline with too few points",
            json!({ "type": "sparkline", "value": {
                "charttype": "line", "data": [point], "options": [] } }),
        ),
    ]
}

#[test]
fn an_undecodable_binding_is_reported_not_dropped() {
    for (what, binding) in undecodable() {
        let result = evaluate("=TYPE(a)", json!({ "a": binding }));
        let err = rejection(&result, "a");
        assert!(
            result.get("value").is_none(),
            "{what}: a rejected binding must not also yield a value: {result}"
        );
        assert!(!err.is_empty(), "{what}: empty error text");
    }
}

/// The same binding shared by every formula in the batch, so one bad binding
/// fails the call rather than being dropped for all of them.
#[test]
fn batch_evaluate_reports_an_undecodable_binding_too() {
    let result = call(
        "batch_evaluate",
        json!({ "formulas": ["=SUM(a)", "=SUM(a, 10)"], "variables": { "a": null } }),
    );
    rejection(&result, "a");
    assert!(
        result.as_array().is_none(),
        "a rejected binding must not produce per-formula results: {result}"
    );
}

/// The rejection must be about the binding that is bad, not the first one seen.
#[test]
fn the_error_names_the_binding_that_is_bad() {
    let result = evaluate("=SUM(good, bad)", json!({ "good": 5, "bad": null }));
    let err = rejection(&result, "bad");
    assert!(
        !err.contains("good"),
        "the error must not blame a binding that decoded fine, got {err:?}"
    );
}

/// Scalars are unaffected — the fix rejects what it cannot decode, and nothing
/// more. Without this, "reject everything" would pass the tests above.
#[test]
fn scalar_bindings_still_evaluate() {
    for (formula, vars, expected) in [
        ("=SUM(s, 10)", json!({ "s": 5 }), json!(15.0)),
        ("=UPPER(t)", json!({ "t": "ab" }), json!("AB")),
        ("=IF(b, 1, 2)", json!({ "b": true }), json!(1.0)),
    ] {
        let result = evaluate(formula, vars);
        assert_eq!(result["value"], expected, "{formula} gave {result}");
    }
}

/// A well-formed self-describing binding still decodes; the rejection path must
/// not have swallowed the shapes the server does support.
#[test]
fn a_zoned_binding_still_decodes() {
    let result = evaluate(
        "=TYPE(z)",
        json!({ "z": { "type": "zoned", "value": "2024-01-01T00:00:00Z[UTC]" } }),
    );
    assert_eq!(result["value"], json!(1.0), "got {result}");
}
