use super::super::*;
use crate::types::Value;

#[test]
fn var_p_basic() {
    // [2, 4, 6]: mean=4, var=((2-4)²+(4-4)²+(6-4)²)/3=8/3
    let result = var_p_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    if let Value::Number(v) = result {
        assert!((v - 8.0 / 3.0).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}

#[test]
fn var_p_two_values() {
    // [1, 3]: mean=2, var=((1-2)²+(3-2)²)/2=1
    let result = var_p_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn var_p_single_value_returns_zero() {
    // n=1: mean=5, var=0/1=0
    assert_eq!(var_p_fn(&[Value::Number(5.0)]), Value::Number(0.0));
}

#[test]
fn var_p_ignores_bool_and_text() {
    // Only numbers: [2, 6] → mean=4, var=(4+4)/2=4
    let result = var_p_fn(&[
        Value::Number(2.0),
        Value::Number(6.0),
        Value::Bool(true),
        Value::Text("x".to_string()),
    ]);
    assert_eq!(result, Value::Number(4.0));
}
