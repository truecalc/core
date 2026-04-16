use super::super::*;
use crate::types::Value;

#[test]
fn quartile_inc_q1() {
    // QUARTILE.INC([1,2,3,4,5], 1) = 2.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(quartile_inc_fn(&[arr, Value::Number(1.0)]), Value::Number(2.0));
}

#[test]
fn quartile_inc_q2_is_median() {
    // QUARTILE.INC([1,2,3,4,5], 2) = 3.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(quartile_inc_fn(&[arr, Value::Number(2.0)]), Value::Number(3.0));
}

#[test]
fn quartile_inc_q0_is_min() {
    // QUARTILE.INC([1,2,3,4,5], 0) = 1.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(quartile_inc_fn(&[arr, Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn quartile_inc_q4_is_max() {
    // QUARTILE.INC([1,2,3,4,5], 4) = 5.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(quartile_inc_fn(&[arr, Value::Number(4.0)]), Value::Number(5.0));
}
