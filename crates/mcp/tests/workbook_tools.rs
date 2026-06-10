use serde_json::{json, Value as JsonValue};
use std::io::Write;

/// Run a sequence of JSON-RPC requests against a single MCP server process.
/// Returns the parsed inner tool payloads (the `text` content of each
/// tools/call response, already parsed as JSON) in the same order.
fn run_session(requests: &[JsonValue]) -> Vec<JsonValue> {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_truecalc-mcp"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start truecalc-mcp");

    let stdin = child.stdin.as_mut().expect("stdin");
    for req in requests {
        writeln!(stdin, "{}", serde_json::to_string(req).unwrap()).unwrap();
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let resp: JsonValue = serde_json::from_str(line).expect("json response");
            // tools/call responses wrap the result in content[0].text
            if let Some(text) = resp["result"]["content"][0]["text"].as_str() {
                serde_json::from_str(text).expect("inner json")
            } else {
                resp["result"].clone()
            }
        })
        .collect()
}

fn call(id: u64, tool: &str, args: JsonValue) -> JsonValue {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": { "name": tool, "arguments": args }
    })
}

/// Canonical workbook JSON with one sheet "Sheet1" and no cells.
fn empty_workbook_json() -> String {
    r#"{"engine":"sheets","names":[],"sheets":[{"cells":{},"name":"Sheet1"}],"version":"1"}"#.to_owned()
}

#[test]
fn workbook_create_returns_id() {
    let results = run_session(&[
        call(1, "workbook_create", json!({ "engine": "sheets" })),
    ]);
    let result = &results[0];
    assert!(result.get("workbook_id").is_some(), "expected workbook_id, got: {result}");
    let id = result["workbook_id"].as_str().unwrap();
    assert!(id.starts_with("wb_"), "id should start with wb_, got: {id}");
}

#[test]
fn workbook_create_excel_engine() {
    let results = run_session(&[
        call(1, "workbook_create", json!({ "engine": "excel" })),
    ]);
    assert!(results[0].get("workbook_id").is_some(), "expected workbook_id, got: {}", results[0]);
}

#[test]
fn workbook_set_recalc_get_sum_formula() {
    // Import a workbook with Sheet1, set cells A1/B1, recalc, then get A1.
    let results = run_session(&[
        call(1, "workbook_import", json!({ "json": empty_workbook_json() })),
        call(2, "workbook_set", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "B1", "value": "3" })),
        call(3, "workbook_set", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "C1", "value": "4" })),
        call(4, "workbook_set", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "A1", "value": "=SUM(B1,C1)" })),
        call(5, "workbook_recalc", json!({ "workbook_id": "wb_0" })),
        call(6, "workbook_get", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "A1" })),
    ]);

    // Import should succeed
    assert!(results[0].get("workbook_id").is_some(), "import: {}", results[0]);

    // Three set calls should succeed
    assert_eq!(results[1]["ok"], json!(true), "set B1: {}", results[1]);
    assert_eq!(results[2]["ok"], json!(true), "set C1: {}", results[2]);
    assert_eq!(results[3]["ok"], json!(true), "set A1: {}", results[3]);

    // Recalc should report A1 changed from empty to 7
    let changes = results[4]["changes"].as_array().expect("changes array");
    let a1_change = changes.iter().find(|c| c["cell"] == "A1").expect("A1 in changes");
    assert_eq!(a1_change["before"]["type"], "empty", "before: {}", a1_change);
    assert_eq!(a1_change["after"]["type"], "number", "after type: {}", a1_change);
    assert_eq!(a1_change["after"]["value"], json!(7.0), "after value: {}", a1_change);

    // Get should return 7
    assert_eq!(results[5]["type"], "number", "get type: {}", results[5]);
    assert_eq!(results[5]["value"], json!(7.0), "get value: {}", results[5]);
}

#[test]
fn workbook_export_import_roundtrip() {
    // Import a bare workbook, set a cell, export, re-import, and read back.
    let results = run_session(&[
        call(1, "workbook_import", json!({ "json": empty_workbook_json() })),
        call(2, "workbook_set", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "A1", "value": "42" })),
        call(3, "workbook_export", json!({ "workbook_id": "wb_0" })),
    ]);

    assert!(results[0].get("workbook_id").is_some());
    assert_eq!(results[1]["ok"], json!(true));
    let exported = results[2]["json"].as_str().expect("exported json string").to_owned();

    // Re-import the exported JSON and read A1
    let results2 = run_session(&[
        call(1, "workbook_import", json!({ "json": &exported })),
        call(2, "workbook_get", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "A1" })),
    ]);
    assert!(results2[0].get("workbook_id").is_some());
    assert_eq!(results2[1]["type"], "number", "get result: {}", results2[1]);
    assert_eq!(results2[1]["value"], json!(42.0));
}

#[test]
fn workbook_session_limit_enforced() {
    // Create 33 workbooks; the 33rd must return a "session limit" error.
    let requests: Vec<JsonValue> = (0..33_u64)
        .map(|i| call(i, "workbook_create", json!({ "engine": "sheets" })))
        .collect();

    let results = run_session(&requests);
    assert_eq!(results.len(), 33);

    // First 32 must succeed
    for r in results.iter().take(32) {
        assert!(r.get("workbook_id").is_some(), "expected workbook_id, got: {r}");
    }

    // 33rd must be an error containing "session limit"
    let last = &results[32];
    assert!(last.get("error").is_some(), "expected error for 33rd workbook, got: {last}");
    assert!(
        last["error"].as_str().unwrap_or("").contains("session limit"),
        "expected 'session limit' in error, got: {}",
        last["error"]
    );
}
