use super::super::*;
use crate::types::Value;

#[test]
fn stdev_p_all_same_values_returns_zero() {
    assert_eq!(
        stdev_p_fn(&[Value::Number(4.0), Value::Number(4.0), Value::Number(4.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn stdev_p_negative_numbers() {
    // [-3, -1]: pop var=1, stdev=1.0
    let result = stdev_p_fn(&[Value::Number(-3.0), Value::Number(-1.0)]);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn stdev_p_large_spread() {
    // [0, 10]: pop var=25, stdev=5.0
    let result = stdev_p_fn(&[Value::Number(0.0), Value::Number(10.0)]);
    assert_eq!(result, Value::Number(5.0));
}
