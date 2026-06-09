use super::super::*;
use crate::types::Value;

#[test]
fn negative_float() {
    let args = [Value::Number(-5.5)];
    assert_eq!(to_pure_number_fn(&args), Value::Number(-5.5));
}

#[test]
fn bool_true_passes_through() {
    // Google Sheets: TO_PURE_NUMBER(TRUE) returns TRUE (boolean passthrough).
    let args = [Value::Bool(true)];
    assert_eq!(to_pure_number_fn(&args), Value::Bool(true));
}

#[test]
fn bool_false_passes_through() {
    // Google Sheets: TO_PURE_NUMBER(FALSE) returns FALSE (boolean passthrough).
    let args = [Value::Bool(false)];
    assert_eq!(to_pure_number_fn(&args), Value::Bool(false));
}

#[test]
fn number_passthrough_simulates_to_dollars() {
    // TO_DOLLARS(42) returns Number(42), so TO_PURE_NUMBER(Number(42)) = Number(42)
    let args = [Value::Number(42.0)];
    assert_eq!(to_pure_number_fn(&args), Value::Number(42.0));
}

#[test]
fn number_passthrough_simulates_to_percent() {
    // TO_PERCENT(0.5) returns Number(0.5), so TO_PURE_NUMBER(Number(0.5)) = Number(0.5)
    let args = [Value::Number(0.5)];
    assert_eq!(to_pure_number_fn(&args), Value::Number(0.5));
}
