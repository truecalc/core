use super::super::*;
use crate::types::Value;

#[test]
fn stdevp_all_same_values_returns_zero() {
    assert_eq!(
        stdevp_fn(&[Value::Number(4.0), Value::Number(4.0), Value::Number(4.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn stdevp_ignores_bool_and_text() {
    // Text/bool ignored; [2, 6] → pop var=4, stdev=2
    let result = stdevp_fn(&[
        Value::Number(2.0),
        Value::Number(6.0),
        Value::Bool(true),
        Value::Text("x".to_string()),
    ]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn stdevp_negative_numbers() {
    // [-3, -1]: pop var=1, stdev=1.0
    let result = stdevp_fn(&[Value::Number(-3.0), Value::Number(-1.0)]);
    assert_eq!(result, Value::Number(1.0));
}
