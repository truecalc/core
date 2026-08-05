use super::super::*;
use crate::types::Value;

#[test]
fn percentrank_exc_middle_value() {
    // PERCENTRANK.EXC([1,2,3,4,5], 3) = 3/6 = 0.5
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_exc_fn(&[arr, Value::Number(3.0)]), Value::Number(0.5));
}

#[test]
fn percentrank_exc_min_value() {
    // PERCENTRANK.EXC([1,2,3,4,5], 1) = 1/6 ≈ 0.167 (3 sig)
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_exc_fn(&[arr, Value::Number(1.0)]), Value::Number(0.167));
}

#[test]
fn percentrank_exc_max_value() {
    // PERCENTRANK.EXC([1,2,3,4,5], 5) = 5/6 ≈ 0.833 (3 sig)
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_exc_fn(&[arr, Value::Number(5.0)]), Value::Number(0.833));
}

#[test]
fn percentrank_exc_custom_significance() {
    // PERCENTRANK.EXC([1,2,3,4,5], 2, 2) = 2/6=0.333..., rounded to 2 sig = 0.33
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_exc_fn(&[arr, Value::Number(2.0), Value::Number(2.0)]), Value::Number(0.33));
}
