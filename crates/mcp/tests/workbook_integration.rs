//! MCP stdio integration tests + workbook determinism leg (#546).
//!
//! These tests spawn the truecalc-mcp binary over stdin/stdout and verify
//! that the MCP surface is byte-identical to the Rust-native workbook API.

use serde_json::{json, Value as JsonValue};
use std::io::Write;
use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet};

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Spawn a single MCP process, send `requests` over stdin, collect responses.
/// Returns the parsed inner tool payloads (content[0].text, already parsed).
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
    r#"{"engine":"sheets","names":[],"sheets":[{"cells":{},"name":"Sheet1"}],"version":"1"}"#
        .to_owned()
}

/// Fixed RecalcContext for deterministic tests.
fn fixed_ctx() -> RecalcContext {
    RecalcContext::new(1_780_000_000_000, "UTC", 42).expect("UTC is valid")
}

fn a1(s: &str) -> Address {
    Address::from_a1(s).expect("valid A1")
}

// ─── Tests ───────────────────────────────────────────────────────────────────

/// Verify the full import → set → recalc → get flow via MCP stdio.
#[test]
fn mcp_import_set_recalc_get() {
    let results = run_session(&[
        call(1, "workbook_import", json!({ "json": empty_workbook_json() })),
        call(2, "workbook_set", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "A1", "value": "=10+20" })),
        call(3, "workbook_recalc", json!({ "workbook_id": "wb_0", "timestamp_ms": 0, "timezone": "UTC", "rng_seed": 0 })),
        call(4, "workbook_get", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "A1" })),
    ]);

    assert!(results[0].get("workbook_id").is_some(), "import: {}", results[0]);
    assert_eq!(results[1]["ok"], json!(true), "set: {}", results[1]);

    // Recalc must report A1 changed empty → 30
    let changes = results[2]["changes"].as_array().expect("changes");
    let a1_change = changes.iter().find(|c| c["cell"] == "A1").expect("A1 in changes");
    assert_eq!(a1_change["before"]["type"], "empty", "before: {a1_change}");
    assert_eq!(a1_change["after"]["type"], "number", "after type: {a1_change}");
    assert_eq!(a1_change["after"]["value"], json!(30.0), "after value: {a1_change}");

    // Get must return 30
    assert_eq!(results[3]["type"], "number", "get type: {}", results[3]);
    assert_eq!(results[3]["value"], json!(30.0), "get value: {}", results[3]);
}

/// Export then re-import returns identical cell data.
#[test]
fn mcp_export_import_roundtrip() {
    let results = run_session(&[
        call(1, "workbook_import", json!({ "json": empty_workbook_json() })),
        call(2, "workbook_set", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "B2", "value": "99" })),
        call(3, "workbook_export", json!({ "workbook_id": "wb_0" })),
    ]);

    assert!(results[0].get("workbook_id").is_some());
    assert_eq!(results[1]["ok"], json!(true));
    let exported = results[2]["json"].as_str().expect("exported json").to_owned();

    // Re-import in a fresh process and read back B2
    let results2 = run_session(&[
        call(1, "workbook_import", json!({ "json": &exported })),
        call(2, "workbook_get", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "B2" })),
    ]);
    assert!(results2[0].get("workbook_id").is_some());
    assert_eq!(results2[1]["type"], "number", "get: {}", results2[1]);
    assert_eq!(results2[1]["value"], json!(99.0));
}

/// DETERMINISM: workbook_import → workbook_recalc → workbook_export via MCP
/// must produce the same canonical JSON as the equivalent Rust-native calls.
#[test]
fn mcp_export_matches_rust_native_export() {
    // ── Step 1: build the canonical JSON via Rust natively ──────────────────
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=B1+C1".into())).unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Literal(Value::Number(7.0))).unwrap();
    wb.set("Sheet1", a1("C1"), CellInput::Literal(Value::Number(8.0))).unwrap();
    wb.recalc(&fixed_ctx());
    let rust_json = wb.to_json().expect("to_json");

    // ── Step 2: reproduce the same state via MCP and export ─────────────────
    // Import the pre-recalc workbook (no-cells base), then set cells and recalc.
    let results = run_session(&[
        call(1, "workbook_import", json!({ "json": empty_workbook_json() })),
        call(2, "workbook_set", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "B1", "value": "7" })),
        call(3, "workbook_set", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "C1", "value": "8" })),
        call(4, "workbook_set", json!({ "workbook_id": "wb_0", "sheet": "Sheet1", "cell": "A1", "value": "=B1+C1" })),
        call(5, "workbook_recalc", json!({ "workbook_id": "wb_0", "timestamp_ms": 1_780_000_000_000_i64, "timezone": "UTC", "rng_seed": 42 })),
        call(6, "workbook_export", json!({ "workbook_id": "wb_0" })),
    ]);

    let mcp_json = results[5]["json"].as_str().expect("mcp export json").to_owned();

    // ── Step 3: byte-for-byte comparison ────────────────────────────────────
    assert_eq!(
        mcp_json, rust_json,
        "MCP export must be byte-identical to Rust-native export.\n  mcp:  {mcp_json}\n  rust: {rust_json}"
    );
}

/// DETERMINISM: two independent Rust-native workbooks built with identical
/// operations produce byte-identical canonical JSON.
#[test]
fn rust_native_exports_are_deterministic() {
    let build = || {
        let mut wb = Workbook::new(EngineFlavor::Sheets);
        wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
        wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(3.0))).unwrap();
        wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1*2".into())).unwrap();
        wb.recalc(&fixed_ctx());
        wb.to_json().expect("to_json")
    };

    let json_a = build();
    let json_b = build();

    assert_eq!(json_a, json_b, "identical Rust workbooks must export identical JSON");
}

/// Session limit: the 33rd workbook in one process must be refused with a
/// "session limit" error.
#[test]
fn mcp_session_limit() {
    let requests: Vec<JsonValue> = (0..33_u64)
        .map(|i| call(i, "workbook_create", json!({ "engine": "sheets" })))
        .collect();

    let results = run_session(&requests);
    assert_eq!(results.len(), 33);

    for r in results.iter().take(32) {
        assert!(r.get("workbook_id").is_some(), "expected workbook_id: {r}");
    }

    let last = &results[32];
    assert!(last.get("error").is_some(), "expected error for 33rd: {last}");
    assert!(
        last["error"].as_str().unwrap_or("").contains("session limit"),
        "expected 'session limit' in error, got: {}",
        last["error"]
    );
}
