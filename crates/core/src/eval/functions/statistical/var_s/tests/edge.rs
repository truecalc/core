use super::super::*;
use crate::types::Value;

#[test]
fn var_s_all_same_values_returns_zero() {
    assert_eq!(
        var_s_fn(&[Value::Number(4.0), Value::Number(4.0), Value::Number(4.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn var_s_negative_numbers() {
    // [-3, -1]: mean=-2, var=((-3-(-2))²+(-1-(-2))²)/1 = (1+1)/1 = 2
    let result = var_s_fn(&[Value::Number(-3.0), Value::Number(-1.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn var_s_large_spread() {
    // [0, 10]: mean=5, var=((0-5)²+(10-5)²)/1 = 50
    let result = var_s_fn(&[Value::Number(0.0), Value::Number(10.0)]);
    assert_eq!(result, Value::Number(50.0));
}
