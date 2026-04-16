use super::super::*;
use crate::types::Value;

#[test]
fn percentile_50th() {
    // PERCENTILE([1,2,3,4,5], 0.5) = 3.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentile_fn(&[arr, Value::Number(0.5)]), Value::Number(3.0));
}

#[test]
fn percentile_0th_is_min() {
    // PERCENTILE([1,2,3,4,5], 0.0) = 1.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentile_fn(&[arr, Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn percentile_100th_is_max() {
    // PERCENTILE([1,2,3,4,5], 1.0) = 5.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentile_fn(&[arr, Value::Number(1.0)]), Value::Number(5.0));
}

#[test]
fn percentile_25th() {
    // PERCENTILE([1,2,3,4,5], 0.25) = 2.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentile_fn(&[arr, Value::Number(0.25)]), Value::Number(2.0));
}
