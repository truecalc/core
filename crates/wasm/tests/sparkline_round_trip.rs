//! A sparkline handed back in as a variable must arrive as the same value.
//!
//! `value_to_result` emits `{ type: "sparkline", value: {...} }`; without the
//! matching decode in `json_to_value` that object matches no branch and falls
//! through to `empty` *silently*, so `TYPE(x)` would answer 1 instead of 128
//! and `ISBLANK(x)` would answer TRUE. This pins the contract `json_to_value`'s
//! own doc comment states.

use truecalc_core::types::{SparklineChartType, SparklineSpec, SparklineValue};
use truecalc_core::Value;
use truecalc_wasm::{json_to_value, value_to_result};

fn round_trip(value: Value) -> Value {
    let emitted = serde_json::to_value(value_to_result(value)).expect("EvalResult serializes");
    json_to_value(&emitted)
}

fn sparkline() -> Value {
    Value::Sparkline(Box::new(SparklineSpec {
        chart_type: SparklineChartType::Column,
        data: vec![
            SparklineValue::number(1.0),
            SparklineValue::Text("a".to_owned()),
            SparklineValue::Blank,
            SparklineValue::Bool(true),
        ],
        options: vec![
            ("color".to_owned(), SparklineValue::Text("red".to_owned())),
            ("ymin".to_owned(), SparklineValue::number(0.0)),
        ],
    }))
}

#[test]
fn a_sparkline_survives_the_emit_then_read_back_round_trip() {
    let original = sparkline();
    let back = round_trip(original.clone());
    match (&original, &back) {
        (Value::Sparkline(a), Value::Sparkline(b)) => assert_eq!(a, b, "the spec must survive"),
        _ => panic!("a sparkline must not read back as {back:?}"),
    }
}

#[test]
fn a_read_back_sparkline_is_still_a_sparkline_to_the_engine() {
    // The failure this guards is silent: `empty` evaluates fine, it just lies.
    let back = round_trip(sparkline());
    assert!(
        matches!(back, Value::Sparkline(_)),
        "read back as {back:?}, so TYPE() would answer 1 instead of 128"
    );
}

#[test]
fn a_malformed_sparkline_object_still_falls_back_to_empty() {
    // Unchanged contract for anything that is not a well-formed spec — and the
    // decoder accepts exactly what the engine can emit, nothing wider: a `data`
    // array shorter than two points is `#N/A` from the evaluator, and
    // `charttype` is always lifted out of the option list.
    let point = serde_json::json!({ "type": "number", "value": 1.0 });
    for bad in [
        // Two valid points, so the unknown charttype is what rejects this —
        // with an empty `data` the length guard would fire first and this row
        // would pass with the charttype check deleted.
        serde_json::json!({ "type": "sparkline", "value": {
            "charttype": "bogus",
            "data": [point.clone(), point.clone()],
            "options": [] } }),
        serde_json::json!({ "type": "sparkline", "value": {
            "charttype": "line", "data": [], "options": [] } }),
        serde_json::json!({ "type": "sparkline", "value": {
            "charttype": "line", "data": [point.clone()], "options": [] } }),
        serde_json::json!({ "type": "sparkline", "value": {
            "charttype": "line",
            "data": [point.clone(), point.clone()],
            "options": [["charttype", { "type": "text", "value": "bar" }]] } }),
        // An option that is not a [key, value] pair. Without the length guard
        // this indexes out of bounds — a panic, not a rejection.
        serde_json::json!({ "type": "sparkline", "value": {
            "charttype": "line",
            "data": [point.clone(), point.clone()],
            "options": [["color"]] } }),
        serde_json::json!({ "type": "sparkline", "value": {
            "charttype": "line",
            "data": [point.clone(), point],
            "options": [["color", { "type": "text", "value": "red" }, "extra"]] } }),
        // A data point whose payload does not match its own tag.
        serde_json::json!({ "type": "sparkline", "value": {
            "charttype": "line",
            "data": [point.clone(), { "type": "number", "value": "not a number" }],
            "options": [] } }),
        serde_json::json!({ "type": "sparkline", "value": {
            "charttype": "line",
            "data": [point.clone(), { "type": "unknown", "value": 1.0 }],
            "options": [] } }),
    ] {
        assert_eq!(json_to_value(&bad), Value::Empty, "{bad}");
    }
}
