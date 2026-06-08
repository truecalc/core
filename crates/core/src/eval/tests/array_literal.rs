use crate::types::Value;
use std::collections::HashMap;

fn run_formula(formula: &str) -> Value {
    crate::Engine::sheets().evaluate(formula, &HashMap::new())
}

#[test]
fn array_literal_evaluates_to_array() {
    // P1.4 (issue #526): array values flow through evaluation unspilled —
    // the engine returns the full array; spilling (or collapsing to the
    // top-left element for a single-cell view) is the workbook/surface
    // layer's job, not the evaluator's.
    let result = run_formula("={1,2,3}");
    assert!(matches!(result, Value::Array(_)));
}

#[test]
fn array_literal_values_are_correct() {
    let result = run_formula("={1,2,3}");
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])
    );
}

#[test]
fn array_literal_empty() {
    let result = run_formula("={}");
    assert!(matches!(result, Value::Array(ref v) if v.is_empty()));
}

#[test]
fn array_literal_mixed_types() {
    let result = run_formula("={1,\"hello\",TRUE}");
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Number(1.0),
            Value::Text("hello".to_string()),
            Value::Bool(true),
        ])
    );
}

#[test]
fn array_literal_in_function_does_not_panic() {
    // SUM with an array arg returns #VALUE! today — that's expected
    // The important thing is no panic.
    let result = run_formula("=SUM({1,2,3})");
    // Either a Value or an Error is fine; just must not panic.
    let _ = result;
}
