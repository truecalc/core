use super::super::*;
use crate::types::Value;

#[test]
fn percentrank_middle_value() {
    // PERCENTRANK([1,2,3,4,5], 3) = 0.5 (count_below=2, n-1=4, 2/4=0.5)
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_fn(&[arr, Value::Number(3.0)]), Value::Number(0.5));
}

#[test]
fn percentrank_min_value() {
    // PERCENTRANK([1,2,3,4,5], 1) = 0.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_fn(&[arr, Value::Number(1.0)]), Value::Number(0.0));
}

#[test]
fn percentrank_max_value() {
    // PERCENTRANK([1,2,3,4,5], 5) = 1.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_fn(&[arr, Value::Number(5.0)]), Value::Number(1.0));
}

#[test]
fn percentrank_custom_significance() {
    // PERCENTRANK([1,2,3,4,5], 2, 2) = 0.25, rounded to 2 sig digits = 0.25
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_fn(&[arr, Value::Number(2.0), Value::Number(2.0)]), Value::Number(0.25));
}
