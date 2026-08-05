//! The sparkline value on the workbook wire: the parsed spec is carried in
//! full, and it is part of *storage* identity.
//!
//! Sheets keeps two notions of sameness. The `=` operator reports any two
//! sparklines equal whatever they plot — that is the engine's
//! `truecalc_core::Value` equality, pinned in `crates/core/tests/sparkline.rs`.
//! `COUNTUNIQUE` nonetheless counts two different sparklines as 2, so the spec
//! is retained and distinguishable, and the storage layer uses that deeper
//! notion: recalc only writes a recomputed cell back when it differs from the
//! stored one.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use truecalc_core::types::{SparklineChartType, SparklineSpec, SparklineValue};
use truecalc_workbook::Value;

fn spec(
    chart_type: SparklineChartType,
    data: Vec<SparklineValue>,
    options: Vec<(String, SparklineValue)>,
) -> Value {
    Value::Sparkline(Box::new(SparklineSpec {
        chart_type,
        data,
        options,
    }))
}

fn line(data: Vec<SparklineValue>) -> Value {
    spec(SparklineChartType::Line, data, Vec::new())
}

fn nums(ns: &[f64]) -> Vec<SparklineValue> {
    ns.iter().copied().map(SparklineValue::number).collect()
}

fn hash_of(v: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}

fn round_trip(v: &Value) -> Value {
    serde_json::from_str(&serde_json::to_string(v).unwrap()).unwrap()
}

#[test]
fn encoding_carries_the_whole_spec_with_canonical_key_order() {
    let value = spec(
        SparklineChartType::Column,
        vec![
            SparklineValue::number(1.0),
            SparklineValue::Blank,
            SparklineValue::Text("a".to_owned()),
        ],
        vec![("color".to_owned(), SparklineValue::Text("red".to_owned()))],
    );
    assert_eq!(
        serde_json::to_string(&value).unwrap(),
        r#"{"type":"sparkline","value":{"charttype":"column","data":[{"type":"number","value":1.0},{"type":"empty","value":null},{"type":"text","value":"a"}],"options":[["color",{"type":"text","value":"red"}]]}}"#
    );
}

#[test]
fn spec_round_trips_through_json() {
    let value = spec(
        SparklineChartType::Winloss,
        vec![
            SparklineValue::number(1.0),
            SparklineValue::number(-1.0),
            SparklineValue::Bool(true),
            SparklineValue::Blank,
        ],
        vec![
            ("ymin".to_owned(), SparklineValue::number(0.0)),
            ("bogus".to_owned(), SparklineValue::Text("x".to_owned())),
        ],
    );
    assert_eq!(round_trip(&value), value);
}

#[test]
fn storage_equality_and_hashing_compare_the_spec() {
    assert_eq!(line(nums(&[1.0, 2.0, 3.0])), line(nums(&[1.0, 2.0, 3.0])));
    assert_eq!(
        hash_of(&line(nums(&[1.0, 2.0, 3.0]))),
        hash_of(&line(nums(&[1.0, 2.0, 3.0])))
    );

    // Different data, different chart type and different options are all
    // different stored values — the payload is never ignored here, even though
    // the `=` operator cannot see it.
    assert_ne!(line(nums(&[1.0, 2.0, 3.0])), line(nums(&[1.0, 2.0, 4.0])));
    assert_ne!(
        line(nums(&[1.0, 2.0])),
        spec(SparklineChartType::Bar, nums(&[1.0, 2.0]), Vec::new())
    );
    assert_ne!(
        line(nums(&[1.0, 2.0])),
        spec(
            SparklineChartType::Line,
            nums(&[1.0, 2.0]),
            vec![("color".to_owned(), SparklineValue::Text("red".to_owned()))]
        )
    );
}

#[test]
fn a_sparkline_is_not_equal_to_any_scalar() {
    assert_ne!(line(nums(&[1.0, 2.0])), Value::Text(String::new()));
    assert_ne!(line(nums(&[1.0, 2.0])), Value::Empty);
    assert_ne!(line(nums(&[1.0, 2.0])), Value::Number(0.0));
}

#[test]
fn malformed_specs_are_rejected_on_decode() {
    let bad = [
        // Unknown charttype.
        r#"{"type":"sparkline","value":{"charttype":"bogus","data":[{"type":"number","value":1.0},{"type":"number","value":2.0}],"options":[]}}"#,
        // A single data point is unrepresentable (the evaluator answers #N/A).
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1.0}],"options":[]}}"#,
        // Missing field.
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1.0},{"type":"number","value":2.0}]}}"#,
        // An option must be a [key, value] pair.
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1.0},{"type":"number","value":2.0}],"options":[["color"]]}}"#,
        // charttype belongs to its own field, not to options.
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1.0},{"type":"number","value":2.0}],"options":[["charttype",{"type":"text","value":"bar"}]]}}"#,
        // Option keys are stored lower-cased.
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1.0},{"type":"number","value":2.0}],"options":[["COLOR",{"type":"text","value":"red"}]]}}"#,
        // A data point must be a scalar cell value.
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1.0},{"type":"array","value":[[{"type":"number","value":1.0},{"type":"number","value":2.0}]]}],"options":[]}}"#,
    ];
    for json in bad {
        assert!(
            serde_json::from_str::<Value>(json).is_err(),
            "should have been rejected: {json}"
        );
    }
}

#[test]
fn a_non_canonical_charttype_is_rejected_on_decode() {
    // `SPARKLINE`'s own argument matching is case-insensitive, so
    // `{"charttype","LINE"}` evaluates; the wire form is canonical-only, as it
    // already is for option keys. Accepting a non-canonical spelling here would
    // load a document the published schema calls malformed.
    for charttype in ["Line", "LINE", "WinLoss", "Bar"] {
        let json = format!(
            r#"{{"type":"sparkline","value":{{"charttype":"{charttype}","data":[{{"type":"number","value":1.0}},{{"type":"number","value":2.0}}],"options":[]}}}}"#
        );
        assert!(
            serde_json::from_str::<Value>(&json).is_err(),
            "should have been rejected: {json}"
        );
    }
}

/// `SparklineSpec` is a public struct, so a spec the evaluator can never
/// produce is constructible. Encoding one would emit bytes that neither the
/// decoder nor the published schema accepts, so the encoder refuses — the same
/// guard `Value::Array` applies to its own shape rules.
#[test]
fn malformed_specs_are_rejected_on_encode() {
    let bad = [
        // `parse_data` answers #REF! for no points and #N/A for one, so neither
        // exists in serialized form.
        spec(SparklineChartType::Line, Vec::new(), Vec::new()),
        spec(SparklineChartType::Line, nums(&[1.0]), Vec::new()),
        spec(
            SparklineChartType::Line,
            nums(&[1.0, 2.0]),
            vec![("COLOR".to_owned(), SparklineValue::Text("red".to_owned()))],
        ),
        spec(
            SparklineChartType::Line,
            nums(&[1.0, 2.0]),
            vec![(
                "charttype".to_owned(),
                SparklineValue::Text("bar".to_owned()),
            )],
        ),
    ];
    for value in bad {
        assert!(
            serde_json::to_string(&value).is_err(),
            "should have been rejected: {value:?}"
        );
    }
}
