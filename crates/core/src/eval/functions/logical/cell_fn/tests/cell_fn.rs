use crate::evaluate;
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}
fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}
fn n(v: f64) -> Value {
    Value::Number(v)
}

// ── type ──────────────────────────────────────────────────────────────────────

#[test]
fn cell_type_of_number_is_v() {
    assert_eq!(run(r#"=CELL("type", 42)"#), t("v"));
}

#[test]
fn cell_type_of_text_is_l() {
    assert_eq!(run(r#"=CELL("type", "hello")"#), t("l"));
}

#[test]
fn cell_type_of_empty_text_is_l() {
    // Empty string literal evaluates to Value::Text("") which matches Text => "l"
    // Value::Empty (blank cell) would give "b", but is not reachable via literal args
    assert_eq!(run(r#"=CELL("type", "")"#), t("l"));
}

// ── contents ─────────────────────────────────────────────────────────────────

#[test]
fn cell_contents_returns_number_value() {
    assert_eq!(run(r#"=CELL("contents", 123)"#), n(123.0));
}

#[test]
fn cell_contents_returns_text_value() {
    assert_eq!(run(r#"=CELL("contents", "hello")"#), t("hello"));
}

// ── col / row ─────────────────────────────────────────────────────────────────

#[test]
fn cell_col_returns_1() {
    assert_eq!(run(r#"=CELL("col", 42)"#), n(1.0));
}

#[test]
fn cell_row_returns_1() {
    assert_eq!(run(r#"=CELL("row", 42)"#), n(1.0));
}

// ── color / width / sheet ─────────────────────────────────────────────────────

#[test]
fn cell_color_returns_0() {
    assert_eq!(run(r#"=CELL("color", 42)"#), n(0.0));
}

#[test]
fn cell_width_returns_8() {
    assert_eq!(run(r#"=CELL("width", 42)"#), n(8.0));
}

#[test]
fn cell_sheet_returns_empty_string() {
    assert_eq!(run(r#"=CELL("sheet", 42)"#), t(""));
}

// ── prefix ────────────────────────────────────────────────────────────────────

#[test]
fn cell_prefix_returns_empty_string() {
    assert_eq!(run(r#"=CELL("prefix", "hello")"#), t(""));
}

// ── address ──────────────────────────────────────────────────────────────────

#[test]
fn cell_address_returns_non_error() {
    // Address returns empty string — just verify no error
    let result = run(r#"=CELL("address", 42)"#);
    assert!(!matches!(result, Value::Error(_)), "expected non-error, got {:?}", result);
}

// ── errors ───────────────────────────────────────────────────────────────────

#[test]
fn cell_invalid_info_type_returns_value_error() {
    assert_eq!(run(r#"=CELL("bogus", 42)"#), Value::Error(ErrorKind::Value));
}

#[test]
fn cell_empty_info_type_returns_value_error() {
    assert_eq!(run(r#"=CELL("", 42)"#), Value::Error(ErrorKind::Value));
}

#[test]
fn cell_numeric_info_type_returns_value_error() {
    assert_eq!(run(r#"=CELL(1, 42)"#), Value::Error(ErrorKind::Value));
}

#[test]
fn cell_one_arg_returns_na() {
    assert_eq!(run(r#"=CELL("type")"#), Value::Error(ErrorKind::NA));
}

#[test]
fn cell_no_args_returns_na() {
    assert_eq!(run("=CELL()"), Value::Error(ErrorKind::NA));
}
