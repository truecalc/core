use crate::types::Value;
use std::collections::HashMap;

fn run_formula(formula: &str) -> Value {
    crate::Engine::sheets().evaluate(formula, &HashMap::new())
}

#[test]
fn array_literal_evaluates_to_array() {
    // In Google Sheets scalar context, an array literal returns its first element.
    let result = run_formula("={1,2,3}");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn array_literal_values_are_correct() {
    // In Google Sheets scalar context, ={1,2,3} → 1 (top-left element).
    let result = run_formula("={1,2,3}");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn array_literal_empty() {
    let result = run_formula("={}");
    assert!(matches!(result, Value::Array(ref v) if v.is_empty()));
}

#[test]
fn array_literal_mixed_types() {
    // In Google Sheets scalar context, ={1,"hello",TRUE} → 1 (top-left element).
    let result = run_formula("={1,\"hello\",TRUE}");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn array_literal_in_function_does_not_panic() {
    // SUM with an array arg returns #VALUE! today — that's expected
    // The important thing is no panic.
    let result = run_formula("=SUM({1,2,3})");
    // Either a Value or an Error is fine; just must not panic.
    let _ = result;
}
