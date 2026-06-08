//! Cell construction invariants, in particular empty-literal rejection
//! (schema spec §4).

use truecalc_workbook::{Cell, Value, WorkbookError};

#[test]
fn literal_cell_holds_value_and_no_formula() {
    let cell = Cell::literal(Value::Number(1.5)).unwrap();
    assert_eq!(cell.value(), &Value::Number(1.5));
    assert_eq!(cell.formula(), None);
}

#[test]
fn empty_literal_is_rejected() {
    assert_eq!(
        Cell::literal(Value::Empty).unwrap_err(),
        WorkbookError::EmptyLiteral
    );
}

#[test]
fn formula_cell_may_hold_empty_value() {
    // A never-evaluated formula cell carries `empty` until first recalc.
    let cell = Cell::with_formula("=A9", Value::Empty);
    assert_eq!(cell.formula(), Some("=A9"));
    assert_eq!(cell.value(), &Value::Empty);
}

#[test]
fn deserializing_empty_literal_is_rejected() {
    let result = serde_json::from_str::<Cell>(r#"{"value":{"type":"empty","value":null}}"#);
    assert!(result.is_err());
}

#[test]
fn deserializing_formula_with_empty_value_is_accepted() {
    let cell: Cell =
        serde_json::from_str(r#"{"formula":"=A9","value":{"type":"empty","value":null}}"#).unwrap();
    assert_eq!(cell.formula(), Some("=A9"));
    assert_eq!(cell.value(), &Value::Empty);
}

#[test]
fn reserved_cell_fields_are_rejected() {
    // `format` and `comment` are reserved names (schema spec §4); unknown
    // fields are rejected in v1 (schema spec §9).
    for json in [
        r#"{"format":{},"value":{"type":"number","value":1}}"#,
        r#"{"comment":"hi","value":{"type":"number","value":1}}"#,
        r#"{"value":{"type":"number","value":1},"spilledFrom":"A1"}"#,
    ] {
        assert!(
            serde_json::from_str::<Cell>(json).is_err(),
            "should reject: {json}"
        );
    }
}

#[test]
fn cell_value_field_is_required() {
    assert!(serde_json::from_str::<Cell>(r#"{"formula":"=1+1"}"#).is_err());
}

#[test]
fn literal_cell_serializes_without_formula_key() {
    let cell = Cell::literal(Value::Number(100.0)).unwrap();
    assert_eq!(
        serde_json::to_string(&cell).unwrap(),
        r#"{"value":{"type":"number","value":100.0}}"#
    );
}

#[test]
fn formula_text_round_trips_verbatim() {
    let cell = Cell::with_formula("=SUM( a1 , 2)  ", Value::Number(3.0));
    let json = serde_json::to_string(&cell).unwrap();
    let back: Cell = serde_json::from_str(&json).unwrap();
    assert_eq!(back.formula(), Some("=SUM( a1 , 2)  "));
}
