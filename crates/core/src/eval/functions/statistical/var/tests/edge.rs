use super::super::*;
use crate::types::Value;

#[test]
fn var_all_same_values_returns_zero() {
    assert_eq!(
        var_fn(&[Value::Number(4.0), Value::Number(4.0), Value::Number(4.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn var_ignores_bool_and_text() {
    // Text/bool ignored; [2, 6] → mean=4, sample var=8
    let result = var_fn(&[
        Value::Number(2.0),
        Value::Number(6.0),
        Value::Bool(true),
        Value::Text("x".to_string()),
    ]);
    assert_eq!(result, Value::Number(8.0));
}
