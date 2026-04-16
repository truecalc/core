use super::super::*;
use crate::types::Value;

#[test]
fn percentrank_inc_middle_value() {
    // PERCENTRANK.INC([1,2,3,4,5], 3) = count_below/n-1 = 2/4 = 0.5
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_inc_fn(&[arr, Value::Number(3.0)]), Value::Number(0.5));
}

#[test]
fn percentrank_inc_min_value() {
    // PERCENTRANK.INC([1,2,3,4,5], 1) = 0/4 = 0.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_inc_fn(&[arr, Value::Number(1.0)]), Value::Number(0.0));
}

#[test]
fn percentrank_inc_max_value() {
    // PERCENTRANK.INC([1,2,3,4,5], 5) = 4/4 = 1.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_inc_fn(&[arr, Value::Number(5.0)]), Value::Number(1.0));
}

#[test]
fn percentrank_inc_with_significance() {
    // PERCENTRANK.INC([1,2,3,4,5], 2, 4) = 0.25 (4 sig digits)
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_inc_fn(&[arr, Value::Number(2.0), Value::Number(4.0)]), Value::Number(0.25));
}
