use super::super::*;
use crate::types::Value;

#[test]
fn vara_true_counts_as_one() {
    // [1.0, 3.0]: mean=2, var=2
    let result = vara_fn(&[Value::Bool(true), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn vara_false_counts_as_zero() {
    // [0.0, 2.0]: mean=1, var=2
    let result = vara_fn(&[Value::Bool(false), Value::Number(2.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn vara_non_parseable_text_counts_as_zero() {
    // AVERAGEA semantics: text -> 0.0; [0.0, 4.0]: sample var=8
    let result = vara_fn(&[Value::Text("hello".to_string()), Value::Number(4.0)]);
    assert_eq!(result, Value::Number(8.0));
}

#[test]
fn vara_all_same_values_returns_zero() {
    assert_eq!(
        vara_fn(&[Value::Number(5.0), Value::Number(5.0), Value::Number(5.0)]),
        Value::Number(0.0)
    );
}
