//! What the server accepted must be visible in the answer, and a tool failure
//! must be a tool failure.
//!
//! Both guard the same failure mode: an operation that reports success while
//! being wrong. A caller who has to pay a round trip to check that its list was
//! not read as text, or that its conformance target was not quietly defaulted,
//! will not check. And `validate` answering `{"valid": false}` is a successful
//! answer to a question, not a failed call — flagging it as one teaches a
//! caller to distrust the flag.

use serde_json::{json, Value as JsonValue};
use std::io::Write;

/// Run one tool call and return the whole `tools/call` result, so the envelope
/// (`isError`) can be inspected alongside the payload.
fn call_raw(name: &str, arguments: JsonValue) -> JsonValue {
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
    response["result"].clone()
}

fn payload(result: &JsonValue) -> JsonValue {
    let text = result["content"][0]["text"].as_str().unwrap_or_default();
    serde_json::from_str(text).expect("inner json")
}

fn call(name: &str, arguments: JsonValue) -> JsonValue {
    payload(&call_raw(name, arguments))
}

/// Drive a whole session over one stdin pipe, returning one payload per request.
fn session(calls: Vec<(&str, JsonValue)>) -> Vec<JsonValue> {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_truecalc-mcp"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start truecalc-mcp");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        for (i, (name, arguments)) in calls.iter().enumerate() {
            let request = json!({
                "jsonrpc": "2.0",
                "id": i + 1,
                "method": "tools/call",
                "params": { "name": name, "arguments": arguments }
            });
            writeln!(stdin, "{}", serde_json::to_string(&request).unwrap()).unwrap();
        }
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let response: JsonValue = serde_json::from_str(l).expect("json");
            payload(&response["result"])
        })
        .collect()
}

fn error_of(result: &JsonValue) -> String {
    result["error"]
        .as_str()
        .unwrap_or_else(|| panic!("expected an error, got {result}"))
        .to_owned()
}

// ── The accepted echo ────────────────────────────────────────────────────────

#[test]
fn evaluate_echoes_the_shape_each_variable_bound_at() {
    let result = call(
        "evaluate",
        json!({ "formula": "=SUM(xs, n)", "variables": { "xs": [1, 2, 3], "n": 4 } }),
    );
    assert_eq!(result["value"], json!(10.0), "got {result}");
    assert_eq!(
        result["accepted"]["bound"],
        json!({ "xs": "array[3]", "n": "number" }),
        "the caller must see a list was read as a list, got {}",
        result["accepted"]
    );
}

#[test]
fn evaluate_echoes_the_conformance_target_it_actually_used() {
    let defaulted = call("evaluate", json!({ "formula": "=SUM(1,2)" }));
    assert_eq!(
        defaulted["accepted"]["conformance"],
        json!("google-sheets"),
        "a defaulted target must be visible without a round trip, got {defaulted}"
    );

    let explicit = call(
        "evaluate",
        json!({ "formula": "=SUM(1,2)", "conformance": "google-sheets" }),
    );
    assert_eq!(explicit["accepted"]["conformance"], json!("google-sheets"));
}

#[test]
fn workbook_set_echoes_the_resolved_cell_and_how_the_value_was_read() {
    let results = session(vec![
        ("workbook_create", json!({ "engine": "sheets" })),
        (
            "workbook_set",
            json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "b2", "value": "007" }),
        ),
        (
            "workbook_get",
            json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "b2" }),
        ),
    ]);

    let set = &results[1];
    assert_eq!(set["ok"], json!(true), "got {set}");
    assert_eq!(
        set["accepted"],
        json!({ "sheet": "Sheet1", "cell": "B2", "as": "number" }),
        "a leading-zero string read as a number is exactly what the echo is for, got {set}"
    );

    let got = &results[2];
    assert_eq!(
        got["accepted"],
        json!({ "sheet": "Sheet1", "cell": "B2" }),
        "got {got}"
    );
}

// ── isError comes from the outcome, not from a key in the payload ────────────

#[test]
fn an_invalid_formula_is_a_successful_answer_from_validate() {
    let result = call_raw("validate", json!({ "formula": "=SUM(" }));
    assert_eq!(
        payload(&result)["valid"],
        json!(false),
        "got {}",
        payload(&result)
    );
    assert!(
        result.get("isError").is_none(),
        "validate answered the question it was asked; it did not fail: {result}"
    );
}

#[test]
fn a_real_tool_failure_is_still_flagged() {
    let result = call_raw("evaluate", json!({ "conformance": "excel" }));
    assert_eq!(
        result["isError"],
        json!(true),
        "a call that could not be carried out must still be flagged, got {result}"
    );
}

// ── No input is silently dropped ─────────────────────────────────────────────

#[test]
fn variables_that_are_not_an_object_are_rejected() {
    // Ignored, this leaves every name unbound, and an unbound name evaluates to
    // empty rather than raising: `=SUM(xs)` answers 0.
    let result = call(
        "evaluate",
        json!({ "formula": "=SUM(xs)", "variables": [1, 2, 3] }),
    );
    let err = error_of(&result);
    assert!(err.contains("variables"), "got {err:?}");
}

#[test]
fn a_formula_that_is_not_a_string_is_rejected() {
    // Coerced to "", this evaluated to an empty value in the middle of an
    // otherwise correct batch — a hole the caller cannot see.
    let result = call("batch_evaluate", json!({ "formulas": ["=SUM(1,2)", 7] }));
    let err = error_of(&result);
    assert!(
        err.contains("formulas") && err.contains('1'),
        "the error must name the offending position, got {err:?}"
    );
}

#[test]
fn a_conformance_target_that_is_not_a_string_is_rejected() {
    // Defaulted, this silently answers with a target the caller did not ask for.
    for tool in ["evaluate", "batch_evaluate"] {
        let args = if tool == "evaluate" {
            json!({ "formula": "=SUM(1,2)", "conformance": 7 })
        } else {
            json!({ "formulas": ["=SUM(1,2)"], "conformance": 7 })
        };
        let err = error_of(&call(tool, args));
        assert!(err.contains("conformance"), "{tool}: got {err:?}");
    }
}

#[test]
fn a_malformed_recalc_context_is_rejected() {
    // Each of these silently defaulted: a bad timezone to UTC, a fractional
    // timestamp to the epoch, a negative seed to 0 — so NOW() and RAND() answer
    // for a context the caller never asked for.
    for bad in [
        json!({ "workbook_id": "wb_0", "timestamp_ms": 1.5 }),
        json!({ "workbook_id": "wb_0", "timezone": 7 }),
        json!({ "workbook_id": "wb_0", "rng_seed": -1 }),
    ] {
        let results = session(vec![
            ("workbook_create", json!({ "engine": "sheets" })),
            ("workbook_recalc", bad.clone()),
        ]);
        let err = error_of(&results[1]);
        assert!(
            err.contains("timestamp_ms") || err.contains("timezone") || err.contains("rng_seed"),
            "the error for {bad} must name the offending argument, got {err:?}"
        );
    }
}
