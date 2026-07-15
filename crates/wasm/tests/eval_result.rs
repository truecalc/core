//! Surface-shape tests for the `EvalResult` mapping (issue #569).
//!
//! These run natively under `cargo nextest`/`cargo test` and assert the exact
//! JSON shape `@truecalc/core` consumers observe. The WASM ABI is not exercised
//! here (that requires a wasm runtime); the value -> result mapping that
//! determines the surface shape is, which is the behavior #569 changes.

use serde_json::json;
use truecalc_core::{ErrorKind, Value};
use truecalc_wasm::{value_to_result, EvalResult};

/// Serialize an `EvalResult` to JSON the way `serde-wasm-bindgen` would for the
/// `type`-tagged shape, so we can assert on the observable consumer payload.
fn shape(value: Value) -> serde_json::Value {
    serde_json::to_value(value_to_result(value)).expect("EvalResult serializes")
}

#[test]
fn number_maps_to_number() {
    assert_eq!(shape(Value::Number(1.5)), json!({ "type": "number", "value": 1.5 }));
}

#[test]
fn text_maps_to_text() {
    assert_eq!(shape(Value::Text("yes".into())), json!({ "type": "text", "value": "yes" }));
}

#[test]
fn bool_maps_to_bool() {
    // Tag stays `bool` (the published <=0.6.x shape), not `boolean`.
    assert_eq!(shape(Value::Bool(true)), json!({ "type": "bool", "value": true }));
}

#[test]
fn empty_maps_to_empty() {
    assert_eq!(shape(Value::Empty), json!({ "type": "empty" }));
}

#[test]
fn error_maps_to_error_with_error_key() {
    assert_eq!(
        shape(Value::Error(ErrorKind::Ref)),
        json!({ "type": "error", "error": "#REF!" })
    );
}

#[test]
fn error_with_message_maps_to_error_with_message_key() {
    // #728: a diagnostic-carrying error surfaces an additive `message` field
    // alongside the unchanged `error` code.
    let msg = "Wrong number of arguments to DATE. Expected 3 arguments, but got 0 arguments.";
    assert_eq!(
        shape(Value::ErrorMsg(ErrorKind::NA, msg.into())),
        json!({ "type": "error", "error": "#N/A", "message": msg })
    );
}

#[test]
fn bare_error_omits_message_key() {
    // #728: messageless errors keep the exact pre-existing shape (no `message`
    // key), so existing consumers are unaffected.
    let out = shape(Value::Error(ErrorKind::DivByZero));
    assert_eq!(out, json!({ "type": "error", "error": "#DIV/0!" }));
    assert!(out.get("message").is_none(), "bare error must not emit a message key");
}

#[test]
fn date_maps_to_distinct_date_not_number() {
    // #569: dates are no longer collapsed to `number`.
    let out = shape(Value::Date(46180.0));
    assert_eq!(out, json!({ "type": "date", "value": 46180.0 }));
    assert_ne!(out["type"], json!("number"));
}

#[test]
fn one_dimensional_array_serializes_each_cell_typed() {
    // #569: arrays no longer map to `{ type: "error", error: "array not supported" }`.
    let v = Value::Array(vec![
        Value::Number(1.0),
        Value::Text("a".into()),
        Value::Bool(false),
    ]);
    assert_eq!(
        shape(v),
        json!({
            "type": "array",
            "value": [
                { "type": "number", "value": 1.0 },
                { "type": "text", "value": "a" },
                { "type": "bool", "value": false }
            ]
        })
    );
}

#[test]
fn two_dimensional_array_nests_rows() {
    // Row-major: outer array of `array` rows whose cells carry their own type.
    let v = Value::Array(vec![
        Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]),
        Value::Array(vec![Value::Number(3.0), Value::Number(4.0)]),
    ]);
    assert_eq!(
        shape(v),
        json!({
            "type": "array",
            "value": [
                { "type": "array", "value": [
                    { "type": "number", "value": 1.0 },
                    { "type": "number", "value": 2.0 }
                ]},
                { "type": "array", "value": [
                    { "type": "number", "value": 3.0 },
                    { "type": "number", "value": 4.0 }
                ]}
            ]
        })
    );
}

#[test]
fn array_preserves_nested_date_and_error_cells() {
    let v = Value::Array(vec![Value::Date(46180.0), Value::Error(ErrorKind::Num)]);
    assert_eq!(
        shape(v),
        json!({
            "type": "array",
            "value": [
                { "type": "date", "value": 46180.0 },
                { "type": "error", "error": "#NUM!" }
            ]
        })
    );
}

#[test]
fn empty_array_serializes_as_empty_value_list() {
    assert_eq!(shape(Value::Array(vec![])), json!({ "type": "array", "value": [] }));
}

/// Exhaustiveness guard: every `Value` variant must produce a non-error result
/// here except the genuine `Error` variant. Guarantees no variant silently
/// falls back to "array not supported"-style errors.
#[test]
fn no_value_variant_maps_to_a_spurious_error() {
    for v in [
        Value::Number(0.0),
        Value::Text(String::new()),
        Value::Bool(false),
        Value::Empty,
        Value::Date(0.0),
        Value::Array(vec![Value::Number(0.0)]),
    ] {
        assert!(
            !matches!(value_to_result(v.clone()), EvalResult::Error { .. }),
            "variant {v:?} unexpectedly mapped to EvalResult::Error"
        );
    }
}
