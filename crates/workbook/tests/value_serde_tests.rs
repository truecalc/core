//! Value wire encodings per schema spec §6 and the non-finite / `-0`
//! policies of §8.

use truecalc_workbook::Value;

fn round_trip(v: &Value) -> Value {
    serde_json::from_str(&serde_json::to_string(v).unwrap()).unwrap()
}

#[test]
fn scalar_encodings_match_schema() {
    let cases = [
        (Value::Number(1.5), r#"{"type":"number","value":1.5}"#),
        (
            Value::Text("yes".to_owned()),
            r#"{"type":"text","value":"yes"}"#,
        ),
        (Value::Boolean(true), r#"{"type":"boolean","value":true}"#),
        (
            Value::Error("#REF!".to_owned()),
            r##"{"error":"#REF!","type":"error"}"##,
        ),
        (Value::Empty, r#"{"type":"empty","value":null}"#),
        (Value::Date(46180.5), r#"{"type":"date","value":46180.5}"#),
    ];
    for (value, expected) in cases {
        assert_eq!(serde_json::to_string(&value).unwrap(), expected);
        assert_eq!(serde_json::from_str::<Value>(expected).unwrap(), value);
    }
}

#[test]
fn error_encoding_uses_error_key_not_value() {
    // `{ "type": "error", "value": ... }` is not the published shape.
    assert!(serde_json::from_str::<Value>(r##"{"type":"error","value":"#REF!"}"##).is_err());
}

#[test]
fn array_round_trips() {
    let array = Value::Array(vec![
        vec![Value::Number(1.0), Value::Number(2.0)],
        vec![Value::Number(3.0), Value::Number(4.0)],
    ]);
    assert_eq!(round_trip(&array), array);
}

#[test]
fn array_may_hold_empty_elements() {
    let array = Value::Array(vec![vec![Value::Empty, Value::Text("x".to_owned())]]);
    assert_eq!(round_trip(&array), array);
}

#[test]
fn nested_arrays_are_rejected() {
    let json =
        r#"{"type":"array","value":[[{"type":"array","value":[[{"type":"number","value":1}]]}]]}"#;
    assert!(serde_json::from_str::<Value>(json).is_err());

    let nested = Value::Array(vec![vec![Value::Array(vec![vec![Value::Number(1.0)]])]]);
    assert!(serde_json::to_string(&nested).is_err());
}

#[test]
fn ragged_and_empty_arrays_are_rejected() {
    for json in [
        r#"{"type":"array","value":[]}"#,
        r#"{"type":"array","value":[[]]}"#,
        r#"{"type":"array","value":[[{"type":"number","value":1}],[{"type":"number","value":2},{"type":"number","value":3}]]}"#,
    ] {
        assert!(
            serde_json::from_str::<Value>(json).is_err(),
            "should reject: {json}"
        );
    }
    assert!(serde_json::to_string(&Value::Array(vec![])).is_err());
    let ragged = Value::Array(vec![
        vec![Value::Number(1.0)],
        vec![Value::Number(2.0), Value::Number(3.0)],
    ]);
    assert!(serde_json::to_string(&ragged).is_err());
}

#[test]
fn non_canonical_key_order_is_accepted() {
    // §8: non-canonical but schema-valid JSON must be accepted.
    assert_eq!(
        serde_json::from_str::<Value>(r#"{"value":1.5,"type":"number"}"#).unwrap(),
        Value::Number(1.5)
    );
    assert_eq!(
        serde_json::from_str::<Value>(r##"{ "type" : "error" , "error" : "#N/A" }"##).unwrap(),
        Value::Error("#N/A".to_owned())
    );
}

#[test]
fn unknown_value_type_is_rejected() {
    assert!(serde_json::from_str::<Value>(r#"{"type":"currency","value":1}"#).is_err());
}

#[test]
fn extra_fields_on_values_are_rejected() {
    for json in [
        r#"{"type":"number","value":1,"unit":"m"}"#,
        r##"{"error":"#REF!","type":"error","value":null}"##,
        r#"{"type":"empty"}"#,
    ] {
        assert!(
            serde_json::from_str::<Value>(json).is_err(),
            "should reject: {json}"
        );
    }
}

#[test]
fn wrong_payload_types_are_rejected() {
    for json in [
        r#"{"type":"number","value":"1"}"#,
        r#"{"type":"text","value":5}"#,
        r#"{"type":"boolean","value":"true"}"#,
        r#"{"type":"empty","value":0}"#,
        r#"{"type":"date","value":"2026-06-07"}"#,
    ] {
        assert!(
            serde_json::from_str::<Value>(json).is_err(),
            "should reject: {json}"
        );
    }
}

#[test]
fn non_finite_numbers_cannot_be_serialized() {
    // §8: serializers must error rather than emit NaN/Infinity.
    assert!(serde_json::to_string(&Value::Number(f64::NAN)).is_err());
    assert!(serde_json::to_string(&Value::Number(f64::INFINITY)).is_err());
    assert!(serde_json::to_string(&Value::Date(f64::NEG_INFINITY)).is_err());
}

#[test]
fn negative_zero_is_normalized_on_deserialization() {
    // §8: the engine normalizes -0 to 0 at the value level.
    let v: Value = serde_json::from_str(r#"{"type":"number","value":-0.0}"#).unwrap();
    assert_eq!(v, Value::Number(0.0));
    assert_eq!(
        serde_json::to_string(&v).unwrap(),
        r#"{"type":"number","value":0.0}"#
    );
}

#[test]
fn negative_zero_is_normalized_on_serialization() {
    assert_eq!(
        serde_json::to_string(&Value::Number(-0.0)).unwrap(),
        r#"{"type":"number","value":0.0}"#
    );
}

#[test]
fn integral_json_numbers_deserialize_as_f64() {
    assert_eq!(
        serde_json::from_str::<Value>(r#"{"type":"number","value":8}"#).unwrap(),
        Value::Number(8.0)
    );
    assert_eq!(
        serde_json::from_str::<Value>(r#"{"type":"date","value":46180}"#).unwrap(),
        Value::Date(46180.0)
    );
}

#[test]
fn one_by_one_arrays_are_rejected() {
    // §6: 1×1 arrays do not exist in serialized form — an operation
    // producing one collapses it to its scalar element before storage.
    let json = r#"{"type":"array","value":[[{"type":"number","value":1}]]}"#;
    assert!(serde_json::from_str::<Value>(json).is_err());
    let one_by_one = Value::Array(vec![vec![Value::Number(1.0)]]);
    assert!(serde_json::to_string(&one_by_one).is_err());
}

#[test]
fn one_by_n_and_n_by_one_arrays_are_accepted() {
    let row = Value::Array(vec![vec![Value::Number(1.0), Value::Number(2.0)]]);
    assert_eq!(round_trip(&row), row);
    let column = Value::Array(vec![vec![Value::Number(1.0)], vec![Value::Number(2.0)]]);
    assert_eq!(round_trip(&column), column);
}

// ── Zoned (Model B): canonical, self-describing RFC-9557 wire form ─────────────

#[test]
fn zoned_iana_round_trips_via_rfc9557() {
    // The serializer must emit exactly the canonical RFC-9557 string, and the
    // value must survive a JSON round trip unchanged.
    let json = r#"{"type":"zoned","value":"2026-07-14T11:00:00+02:00[Europe/Berlin]"}"#;
    let v = serde_json::from_str::<Value>(json).unwrap();
    assert_eq!(serde_json::to_string(&v).unwrap(), json);
    assert_eq!(round_trip(&v), v);
}

#[test]
fn zoned_fixed_offset_round_trips() {
    let json = r#"{"type":"zoned","value":"2026-01-01T12:00:00+05:30"}"#;
    let v = serde_json::from_str::<Value>(json).unwrap();
    assert_eq!(serde_json::to_string(&v).unwrap(), json);
    assert_eq!(round_trip(&v), v);
}

#[test]
fn zoned_rejects_invalid_rfc9557() {
    assert!(serde_json::from_str::<Value>(r#"{"type":"zoned","value":"not a timestamp"}"#).is_err());
}
