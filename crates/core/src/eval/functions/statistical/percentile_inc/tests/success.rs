use super::super::*;
use crate::types::Value;

#[test]
fn percentile_inc_50th() {
    // PERCENTILE.INC([1,2,3,4,5], 0.5) = 3.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentile_inc_fn(&[arr, Value::Number(0.5)]), Value::Number(3.0));
}

#[test]
fn percentile_inc_0th_is_min() {
    // PERCENTILE.INC([1,2,3,4,5], 0.0) = 1.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentile_inc_fn(&[arr, Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn percentile_inc_100th_is_max() {
    // PERCENTILE.INC([1,2,3,4,5], 1.0) = 5.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentile_inc_fn(&[arr, Value::Number(1.0)]), Value::Number(5.0));
}

#[test]
fn percentile_inc_75th() {
    // PERCENTILE.INC([1,2,3,4,5], 0.75) = 4.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentile_inc_fn(&[arr, Value::Number(0.75)]), Value::Number(4.0));
}
