use super::super::*;
use crate::types::Value;

#[test]
fn var_p_all_same_values_returns_zero() {
    assert_eq!(
        var_p_fn(&[Value::Number(4.0), Value::Number(4.0), Value::Number(4.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn var_p_negative_numbers() {
    // [-3, -1]: mean=-2, var=((-3-(-2))²+(-1-(-2))²)/2 = (1+1)/2 = 1
    let result = var_p_fn(&[Value::Number(-3.0), Value::Number(-1.0)]);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn var_p_large_spread() {
    // [0, 10]: mean=5, var=((0-5)²+(10-5)²)/2 = 50/2 = 25
    let result = var_p_fn(&[Value::Number(0.0), Value::Number(10.0)]);
    assert_eq!(result, Value::Number(25.0));
}
