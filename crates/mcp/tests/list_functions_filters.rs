//! `list_functions` must be able to answer a narrow question narrowly.
//!
//! Unfiltered the catalogue is ~70 KB (~17.5k tokens), which is what an agent
//! paid to ask "what is the signature of XLOOKUP". The filters make the cheap
//! question cheap; the unfiltered call still returns the whole catalogue so
//! nothing that already depends on it changes.

use serde_json::{json, Value as JsonValue};
use std::io::Write;

/// Run one `list_functions` call against the binary and return its inner JSON.
fn list_functions(arguments: JsonValue) -> JsonValue {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_truecalc-mcp"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start truecalc-mcp");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": { "name": "list_functions", "arguments": arguments }
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

fn names_of(result: &JsonValue) -> Vec<String> {
    result["functions"]
        .as_array()
        .unwrap_or_else(|| panic!("expected a functions array, got {result}"))
        .iter()
        .map(|e| e["name"].as_str().unwrap_or_default().to_owned())
        .collect()
}

#[test]
fn an_unfiltered_call_still_returns_the_whole_catalogue() {
    let all = list_functions(json!({}));
    let total = all["functions"].as_array().expect("functions").len();
    assert!(
        total > 500,
        "the unfiltered call must stay uncapped, got {total} entries"
    );
    assert_eq!(
        all["total_matched"].as_u64(),
        Some(total as u64),
        "the total must be reported even when nothing was capped, got {}",
        all["total_matched"]
    );
}

#[test]
fn category_returns_only_that_category() {
    let result = list_functions(json!({ "category": "lookup", "limit": 500 }));
    let entries = result["functions"].as_array().expect("functions");
    assert!(
        !entries.is_empty(),
        "expected lookup functions, got {result}"
    );
    for entry in entries {
        assert_eq!(
            entry["category"], "lookup",
            "every entry must be in the requested category, got {entry}"
        );
    }
    assert!(
        entries.len() < 100,
        "a single category must be far smaller than the catalogue, got {}",
        entries.len()
    );
}

#[test]
fn an_unknown_category_is_an_error_not_an_empty_list() {
    let result = list_functions(json!({ "category": "spatial" }));
    let err = result["error"]
        .as_str()
        .unwrap_or_else(|| panic!("expected an error, got {result}"));
    assert!(
        err.contains("spatial") && err.contains("lookup"),
        "the error must name the bad category and the valid ones, got {err:?}"
    );
}

#[test]
fn name_contains_matches_a_substring_case_insensitively() {
    let result = list_functions(json!({ "name_contains": "lookup", "limit": 100 }));
    let names = names_of(&result);
    assert!(
        names.iter().any(|n| n == "XLOOKUP"),
        "expected XLOOKUP among {names:?}"
    );
    for name in &names {
        assert!(
            name.to_ascii_lowercase().contains("lookup"),
            "{name} does not contain the requested substring"
        );
    }
}

#[test]
fn names_looks_up_several_signatures_in_one_call() {
    let result = list_functions(json!({ "names": ["SUM", "xlookup"] }));
    let mut names = names_of(&result);
    names.sort();
    assert_eq!(
        names,
        vec!["SUM".to_owned(), "XLOOKUP".to_owned()],
        "got {result}"
    );
}

#[test]
fn a_name_that_does_not_exist_is_reported_rather_than_dropped() {
    let result = list_functions(json!({ "names": ["SUM", "NOSUCHFN"] }));
    assert_eq!(
        result["not_found"],
        json!(["NOSUCHFN"]),
        "an unmatched name must be named back to the caller, got {result}"
    );
}

#[test]
fn limit_caps_the_page_and_reports_the_full_total() {
    let result = list_functions(json!({ "category": "statistical", "limit": 5 }));
    let entries = result["functions"].as_array().expect("functions");
    assert_eq!(entries.len(), 5, "got {}", entries.len());
    assert_eq!(result["returned"], json!(5), "got {result}");
    let total = result["total_matched"].as_u64().expect("total_matched");
    assert!(
        total > 5,
        "the caller must be able to see it was capped, got total_matched={total}"
    );
}

#[test]
fn a_filter_the_server_does_not_know_is_rejected() {
    // Silently ignoring `fields` would answer a narrow question with the whole
    // 17.5k-token catalogue and no sign that the filter did nothing.
    let result = list_functions(json!({ "fields": ["name"] }));
    let err = result["error"].as_str().unwrap_or_else(|| {
        panic!(
            "expected an error, got a catalogue: {}",
            result["total_matched"]
        )
    });
    assert!(
        err.contains("fields"),
        "the error must name the unsupported argument, got {err:?}"
    );
}

#[test]
fn a_malformed_filter_value_is_rejected() {
    for (bad, needle) in [
        (json!({ "category": 7 }), "category"),
        (json!({ "names": "SUM" }), "names"),
        (json!({ "names": [1] }), "names"),
        (json!({ "limit": 0 }), "limit"),
        (json!({ "limit": "10" }), "limit"),
    ] {
        let result = list_functions(bad.clone());
        let err = result["error"]
            .as_str()
            .unwrap_or_else(|| panic!("expected an error for {bad}, got {result}"));
        assert!(
            err.contains(needle),
            "the error for {bad} must name {needle:?}, got {err:?}"
        );
    }
}
