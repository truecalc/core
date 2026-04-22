use serde_json::json;

fn evaluate(formula: &str, conformance: Option<&str>) -> serde_json::Value {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_truecalc-mcp"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start truecalc-mcp");

    let mut args_obj = json!({ "formula": formula });
    if let Some(c) = conformance {
        args_obj["conformance"] = json!(c);
    }

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": { "name": "evaluate", "arguments": args_obj }
    });

    use std::io::Write;
    let stdin = child.stdin.as_mut().expect("stdin");
    writeln!(stdin, "{}", serde_json::to_string(&request).unwrap()).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let response: serde_json::Value = serde_json::from_str(stdout.trim()).expect("json");
    let text = response["result"]["content"][0]["text"].as_str().unwrap_or("");
    serde_json::from_str(text).expect("inner json")
}

#[test]
fn evaluate_sum_google_sheets() {
    let result = evaluate("=SUM(1,2)", Some("google-sheets"));
    assert_eq!(result["value"], json!(3.0));
    assert_eq!(result["type"], json!("number"));
}

#[test]
fn evaluate_defaults_to_google_sheets() {
    let result = evaluate("=SUM(1,2)", None);
    assert_eq!(result["value"], json!(3.0));
}

#[test]
fn evaluate_unknown_conformance_returns_error() {
    let result = evaluate("=SUM(1,2)", Some("excel"));
    assert!(result.get("error").is_some());
}
