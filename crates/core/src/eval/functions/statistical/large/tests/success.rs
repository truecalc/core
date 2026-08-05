use super::super::*;
use crate::types::Value;

#[test]
fn large_k1_returns_max() {
    // LARGE([1,2,3,4,5], 1) = 5
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(large_fn(&[arr, Value::Number(1.0)]), Value::Number(5.0));
}

#[test]
fn large_k2() {
    // LARGE([1,2,3,4,5], 2) = 4
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(large_fn(&[arr, Value::Number(2.0)]), Value::Number(4.0));
}

#[test]
fn large_single_value_k1() {
    // LARGE([7], 1) = 7
    assert_eq!(
        large_fn(&[Value::Number(7.0), Value::Number(1.0)]),
        Value::Number(7.0)
    );
}

#[test]
fn large_kn_returns_min() {
    // LARGE([1,2,3], 3) = 1
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
    ]);
    assert_eq!(large_fn(&[arr, Value::Number(3.0)]), Value::Number(1.0));
}
