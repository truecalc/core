//! A tool that says what it returns must actually return that.
//!
//! Every tool declares an `inputSchema`; the ones narrowed to the stateless
//! evaluator now also declare an `outputSchema` and answer in
//! `structuredContent`. Nothing at runtime checks that the two agree — this
//! server hand-rolls JSON-RPC and has no SDK to validate its own outbound
//! payload — so this file is the only thing standing between a published
//! contract and a server that quietly stops honouring it.
//!
//! Which makes the method load-bearing: every case below drives the real
//! binary over stdio, reads the declaration off the same `tools/list` a caller
//! reads, and validates the real `structuredContent` against it. Checking a
//! schema against a hand-written fixture would prove only that the fixture
//! matches the schema.
//!
//! `content` is asserted unchanged throughout. `structuredContent` is a new
//! channel, so the absence-carries-meaning repairs it makes (`message`,
//! `error`, `not_found` and `limitApplied` always present) reach nobody who is
//! reading the old one.

use jsonschema::Validator;
use serde_json::{json, Value as JsonValue};
use std::io::Write;

/// The tools that publish an `outputSchema`. The six that survive the
/// narrowing to a stateless evaluator; the `workbook_*` tools deliberately do
/// not declare one yet.
const DECLARED: [&str; 6] = [
    "evaluate",
    "validate",
    "explain",
    "batch_evaluate",
    "list_functions",
    "get_stats",
];

/// Drive one stdio session and return one `tools/call`-style result per
/// request, in order.
fn session(requests: Vec<JsonValue>) -> Vec<JsonValue> {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_truecalc-mcp"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start truecalc-mcp");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        for (i, req) in requests.iter().enumerate() {
            let mut req = req.clone();
            req["jsonrpc"] = json!("2.0");
            req["id"] = json!(i + 1);
            writeln!(
                stdin,
                "{}",
                serde_json::to_string(&req).expect("request json")
            )
            .expect("write");
        }
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let response: JsonValue = serde_json::from_str(l).expect("response json");
            response["result"].clone()
        })
        .collect()
}

fn call_raw(name: &str, arguments: JsonValue) -> JsonValue {
    let mut results = session(vec![
        json!({ "method": "tools/call", "params": { "name": name, "arguments": arguments } }),
    ]);
    results.pop().expect("one result")
}

fn tools_list() -> Vec<JsonValue> {
    let mut results = session(vec![json!({ "method": "tools/list", "params": {} })]);
    let result = results.pop().expect("one result");
    result["tools"].as_array().expect("tools array").clone()
}

fn declaration(name: &str) -> JsonValue {
    tools_list()
        .into_iter()
        .find(|t| t["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("no tool named {name} in tools/list"))
}

/// The JSON a caller reads out of the old channel.
fn content_payload(result: &JsonValue) -> JsonValue {
    let text = result["content"][0]["text"].as_str().expect("content text");
    serde_json::from_str(text).expect("content json")
}

fn structured(result: &JsonValue) -> JsonValue {
    result
        .get("structuredContent")
        .cloned()
        .unwrap_or_else(|| panic!("tools/call result carries no structuredContent: {result}"))
}

fn compile(schema: &JsonValue) -> Validator {
    jsonschema::validator_for(schema).expect("outputSchema does not compile")
}

fn assert_valid(tool: &str, args: &JsonValue, schema: &Validator, instance: &JsonValue) {
    let errors: Vec<String> = schema
        .iter_errors(instance)
        .map(|e| format!("  at {}: {e}", e.instance_path()))
        .collect();
    assert!(
        errors.is_empty(),
        "{tool} {args} returned structuredContent that violates its own outputSchema:\n{}\npayload: {instance}",
        errors.join("\n")
    );
}

/// Every argument set the conformance run exercises, joined to the tool it
/// calls. `evaluate` covers all eight strings `value_to_json` can put in
/// `type`, because that field is the one an agent branches on.
fn fixtures() -> Vec<(&'static str, JsonValue)> {
    vec![
        ("evaluate", json!({ "formula": "=SUM(1,2)" })),
        ("evaluate", json!({ "formula": "=\"hi\"" })),
        ("evaluate", json!({ "formula": "=TRUE()" })),
        ("evaluate", json!({ "formula": "=x" })),
        ("evaluate", json!({ "formula": "=1/0" })),
        ("evaluate", json!({ "formula": "=ABS()" })),
        (
            "evaluate",
            json!({ "formula": "=z", "variables": { "z": { "type": "zoned", "value": "2026-01-02T03:04:05Z[UTC]" } } }),
        ),
        ("evaluate", json!({ "formula": "=SEQUENCE(2)" })),
        ("evaluate", json!({ "formula": "=SPARKLINE({1,2,3})" })),
        ("validate", json!({ "formula": "=SUM(1,2)" })),
        ("validate", json!({ "formula": "=SUM(" })),
        ("explain", json!({ "formula": "=SUM(1,2)" })),
        ("explain", json!({ "formula": "=SUM(" })),
        (
            "batch_evaluate",
            json!({ "formulas": ["=SUM(1,2)", "=ABS()"] }),
        ),
        ("list_functions", json!({})),
        ("list_functions", json!({ "category": "lookup" })),
        ("list_functions", json!({ "names": ["SUM", "NOPE"] })),
        (
            "list_functions",
            json!({ "name_contains": "LOOKUP", "limit": 2 }),
        ),
        ("get_stats", json!({})),
    ]
}

#[test]
fn the_six_evaluator_tools_declare_an_output_schema() {
    let declared: Vec<String> = tools_list()
        .iter()
        .filter(|t| t.get("outputSchema").is_some())
        .map(|t| t["name"].as_str().unwrap_or_default().to_owned())
        .collect();

    assert!(
        !declared.is_empty(),
        "no tool in tools/list declares an outputSchema"
    );
    assert_eq!(
        declared,
        DECLARED.to_vec(),
        "the set of tools declaring an outputSchema changed"
    );
}

#[test]
fn a_declared_schema_is_an_open_object_at_the_root() {
    for name in DECLARED {
        let schema = declaration(name)["outputSchema"].clone();
        // The protocol requires an object root, and a caller pinned to today's
        // schema must not break when a field is added later.
        assert_eq!(
            schema["type"],
            json!("object"),
            "{name}: outputSchema root is not an object"
        );
        assert!(
            schema.get("additionalProperties").is_none(),
            "{name}: outputSchema closes the object, so any future field is a breaking change"
        );
        assert!(
            schema["required"].is_array(),
            "{name}: outputSchema declares no required fields"
        );
        compile(&schema);
    }
}

#[test]
fn declared_tools_answer_with_structured_content_matching_their_schema() {
    for name in DECLARED {
        let schema = compile(&declaration(name)["outputSchema"]);
        let cases: Vec<JsonValue> = fixtures()
            .into_iter()
            .filter(|(t, _)| *t == name)
            .map(|(_, a)| a)
            .collect();
        assert!(
            !cases.is_empty(),
            "{name} declares an outputSchema but has no fixture"
        );
        for args in cases {
            let result = call_raw(name, args.clone());
            assert_valid(name, &args, &schema, &structured(&result));
        }
    }
}

#[test]
fn every_tool_answers_with_structured_content() {
    // Undeclared tools carry it too — it is a new channel, so this costs an
    // existing caller nothing and leaves nothing for the deferred declarations
    // to retrofit.
    for (tool, args) in fixtures() {
        let result = call_raw(tool, args.clone());
        assert!(
            result.get("structuredContent").is_some(),
            "{tool} {args} answered without structuredContent"
        );
    }
    let results = session(vec![
        json!({ "method": "tools/call", "params": { "name": "workbook_create", "arguments": { "engine": "sheets" } } }),
    ]);
    assert!(
        results[0].get("structuredContent").is_some(),
        "workbook_create answered without structuredContent"
    );
}

#[test]
fn the_content_channel_is_byte_identical_to_before() {
    // Pinned against the strings this server emitted before it grew a second
    // channel. An existing caller reads `content` and must see no change.
    let goldens: Vec<(&str, JsonValue, &str)> = vec![
        (
            "evaluate",
            json!({ "formula": "=SUM(1,2)" }),
            r#"{"accepted":{"bound":{},"conformance":"google-sheets"},"type":"number","value":3.0}"#,
        ),
        (
            "validate",
            json!({ "formula": "=SUM(1,2)" }),
            r#"{"valid":true}"#,
        ),
        (
            "explain",
            json!({ "formula": "=SUM(1,2)" }),
            r#"{"description":"Formula using: SUM","functions_used":["SUM"]}"#,
        ),
        (
            "batch_evaluate",
            json!({ "formulas": ["=SUM(1,2)"] }),
            r#"[{"type":"number","value":3.0}]"#,
        ),
        (
            "list_functions",
            json!({ "names": ["SUM"] }),
            r#"{"functions":[{"category":"math","description":"Sum of arguments","name":"SUM","syntax":"SUM(value1,...)"}],"returned":1,"total_matched":1}"#,
        ),
    ];
    for (tool, args, golden) in goldens {
        let result = call_raw(tool, args.clone());
        assert_eq!(
            result["content"][0]["text"].as_str().expect("content text"),
            golden,
            "{tool} {args}: the content channel changed"
        );
    }
}

#[test]
fn a_field_that_carried_meaning_by_being_absent_is_now_always_present() {
    // Each pair below is the same call read through both channels: the old one
    // omits the field, the new one states it.
    let number = call_raw("evaluate", json!({ "formula": "=SUM(1,2)" }));
    assert!(content_payload(&number).get("message").is_none());
    assert_eq!(structured(&number)["message"], JsonValue::Null);

    let with_message = call_raw("evaluate", json!({ "formula": "=ABS()" }));
    assert!(structured(&with_message)["message"]
        .as_str()
        .expect("message")
        .contains("ABS"));

    let valid = call_raw("validate", json!({ "formula": "=SUM(1,2)" }));
    assert!(content_payload(&valid).get("error").is_none());
    assert_eq!(structured(&valid)["error"], JsonValue::Null);

    let invalid = call_raw("validate", json!({ "formula": "=SUM(" }));
    assert!(structured(&invalid)["error"].is_string());

    let all_found = call_raw("list_functions", json!({ "names": ["SUM"] }));
    assert!(content_payload(&all_found).get("not_found").is_none());
    assert_eq!(structured(&all_found)["not_found"], json!([]));

    let one_missing = call_raw("list_functions", json!({ "names": ["SUM", "NOPE"] }));
    assert_eq!(structured(&one_missing)["not_found"], json!(["NOPE"]));
}

#[test]
fn list_functions_says_which_limit_regime_applied() {
    // The cap is chosen by the request — uncapped when unfiltered, 100 when
    // filtered — and nothing in the answer said which rule ran.
    let unfiltered = call_raw("list_functions", json!({}));
    assert_eq!(structured(&unfiltered)["limitApplied"], JsonValue::Null);

    let filtered = call_raw("list_functions", json!({ "category": "lookup" }));
    assert_eq!(structured(&filtered)["limitApplied"], json!(100));

    let explicit = call_raw(
        "list_functions",
        json!({ "category": "lookup", "limit": 2 }),
    );
    assert_eq!(structured(&explicit)["limitApplied"], json!(2));
}

#[test]
fn batch_evaluate_answers_with_an_object_in_the_new_channel() {
    // `outputSchema`'s root must be an object; this tool's `content` is a bare
    // array, so the new channel names the array instead of replacing it.
    let result = call_raw(
        "batch_evaluate",
        json!({ "formulas": ["=SUM(1,2)", "=ABS()"] }),
    );
    assert!(
        content_payload(&result).is_array(),
        "content must stay a bare array"
    );

    let structured = structured(&result);
    let results = structured["results"].as_array().expect("results array");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["message"], JsonValue::Null);
    assert!(results[1]["message"].is_string());
}

#[test]
fn a_failed_call_is_still_flagged_as_one() {
    // Guards the flag against being derived from probing the payload for an
    // "error" key again: nesting or wrapping the payload would make the probe
    // miss and report every failure as a success.
    let result = call_raw("evaluate", json!({}));
    assert_eq!(
        result["isError"],
        json!(true),
        "a call with no formula must be flagged as an error"
    );

    let ok = call_raw("validate", json!({ "formula": "=SUM(" }));
    assert!(
        ok.get("isError").is_none(),
        "an answer of \"this formula does not parse\" is a successful call"
    );
}

#[test]
fn a_failed_call_on_a_declared_tool_carries_no_nonconforming_structured_content() {
    // The exact gap `a_failed_call_is_still_flagged_as_one` leaves: it drives
    // `evaluate {}` to failure but never looks at the payload. A
    // schema-validating MCP client only validates `structuredContent` when the
    // key is present at all (isError does not exempt an existing key from
    // validation) — so a declared tool that fails must either omit
    // `structuredContent` entirely or fill it with something that satisfies
    // its own outputSchema. Emitting the bare `{"error": ...}` envelope
    // (which has neither "type" nor "value" nor "message") is what broke a
    // real SDK client with a thrown protocol error before this test existed.
    //
    // `get_stats` takes no arguments at all, so it has no failure arm to
    // exercise here; the other five declared tools each have one.
    for (name, bad_args) in [
        ("evaluate", json!({})),
        ("batch_evaluate", json!({})),
        ("validate", json!({})),
        ("explain", json!({})),
        ("list_functions", json!({ "limit": -1 })),
    ] {
        let result = call_raw(name, bad_args.clone());
        assert_eq!(
            result["isError"],
            json!(true),
            "{name} {bad_args} should have failed"
        );
        if let Some(sc) = result.get("structuredContent") {
            let schema = compile(&declaration(name)["outputSchema"]);
            assert_valid(name, &bad_args, &schema, sc);
        }
    }
}
