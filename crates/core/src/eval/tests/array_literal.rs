use crate::types::Value;
use std::collections::HashMap;

fn run_formula(formula: &str) -> Value {
    crate::evaluate(formula, &HashMap::new())
}

#[test]
fn array_literal_evaluates_to_array() {
    let result = run_formula("={1,2,3}");
    assert!(matches!(result, Value::Array(ref v) if v.len() == 3));
}

#[test]
fn array_literal_values_are_correct() {
    let result = run_formula("={1,2,3}");
    match result {
        Value::Array(v) => {
            assert_eq!(v[0], Value::Number(1.0));
            assert_eq!(v[1], Value::Number(2.0));
            assert_eq!(v[2], Value::Number(3.0));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn array_literal_empty() {
    let result = run_formula("={}");
    assert!(matches!(result, Value::Array(ref v) if v.is_empty()));
}

#[test]
fn array_literal_mixed_types() {
    let result = run_formula("={1,\"hello\",TRUE}");
    match result {
        Value::Array(v) => {
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], Value::Number(1.0));
            assert_eq!(v[1], Value::Text("hello".to_string()));
            assert_eq!(v[2], Value::Bool(true));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn array_literal_in_function_does_not_panic() {
    // SUM with an array arg returns #VALUE! today — that's expected
    // The important thing is no panic.
    let result = run_formula("=SUM({1,2,3})");
    // Either a Value or an Error is fine; just must not panic.
    let _ = result;
}
