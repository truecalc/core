use super::super::*;
use crate::types::Value;

#[test]
fn small_k1_returns_min() {
    // SMALL([1,2,3,4,5], 1) = 1
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(small_fn(&[arr, Value::Number(1.0)]), Value::Number(1.0));
}

#[test]
fn small_k2() {
    // SMALL([1,2,3,4,5], 2) = 2
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(small_fn(&[arr, Value::Number(2.0)]), Value::Number(2.0));
}

#[test]
fn small_single_value_k1() {
    // SMALL([7], 1) = 7
    assert_eq!(
        small_fn(&[Value::Number(7.0), Value::Number(1.0)]),
        Value::Number(7.0)
    );
}

#[test]
fn small_kn_returns_max() {
    // SMALL([1,2,3], 3) = 3
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
    ]);
    assert_eq!(small_fn(&[arr, Value::Number(3.0)]), Value::Number(3.0));
}
