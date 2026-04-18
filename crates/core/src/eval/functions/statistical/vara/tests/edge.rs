use super::super::*;
use crate::types::Value;

#[test]
fn vara_true_counts_as_one() {
    // Bool(true)=1.0 is included; [1.0, 3.0]: mean=2, var=2
    let result = vara_fn(&[Value::Bool(true), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn vara_false_counts_as_zero() {
    // Bool(false)=0.0 is included; [0.0, 2.0]: mean=1, var=2
    let result = vara_fn(&[Value::Bool(false), Value::Number(2.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn vara_text_returns_value_error() {
    // Literal text as direct arg → #VALUE! (Google Sheets oracle)
    let result = vara_fn(&[Value::Text("hello".to_string()), Value::Number(4.0)]);
    assert_eq!(result, Value::Error(crate::types::ErrorKind::Value));
}

#[test]
fn vara_all_same_values_returns_zero() {
    assert_eq!(
        vara_fn(&[Value::Number(5.0), Value::Number(5.0), Value::Number(5.0)]),
        Value::Number(0.0)
    );
}
