use serde_json::json;

/// Drive the MCP server over stdio and return the parsed inner tool result.
fn call_tool(name: &str, arguments: serde_json::Value) -> serde_json::Value {
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
fn search_functions_ranks_pmt_for_loan_payment() {
    let result = call_tool("search_functions", json!({ "query": "monthly loan payment" }));
    let matches = result["matches"].as_array().expect("matches array");
    assert!(!matches.is_empty(), "expected ranked matches");
    assert_eq!(matches[0]["name"], json!("PMT"));
    // Every match carries a signature so an agent can call it directly.
    assert!(matches[0]["syntax"].as_str().unwrap().starts_with("PMT("));
    assert!(matches[0]["score"].as_u64().unwrap() > 0);
}

#[test]
fn search_functions_respects_limit() {
    let result = call_tool("search_functions", json!({ "query": "value", "limit": 3 }));
    let matches = result["matches"].as_array().expect("matches array");
    assert!(matches.len() <= 3);
}

#[test]
fn search_functions_missing_query_is_error() {
    let result = call_tool("search_functions", json!({}));
    assert!(result.get("error").is_some());
}

#[test]
fn search_functions_appears_in_tools_list() {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_truecalc-mcp"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start truecalc-mcp");

    let request = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });
    use std::io::Write;
    let stdin = child.stdin.as_mut().expect("stdin");
    writeln!(stdin, "{}", serde_json::to_string(&request).unwrap()).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let response: serde_json::Value = serde_json::from_str(stdout.trim()).expect("json");
    let tools = response["result"]["tools"].as_array().expect("tools array");
    assert!(
        tools.iter().any(|t| t["name"] == json!("search_functions")),
        "search_functions not advertised in tools/list"
    );
}
