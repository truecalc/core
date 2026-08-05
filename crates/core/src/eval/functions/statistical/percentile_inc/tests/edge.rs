use super::super::*;
use crate::types::Value;

#[test]
fn percentile_inc_single_value() {
    // PERCENTILE.INC([7], k) = 7.0 for any k
    assert_eq!(percentile_inc_fn(&[Value::Number(7.0), Value::Number(0.5)]), Value::Number(7.0));
}

#[test]
fn percentile_inc_interpolation() {
    // PERCENTILE.INC([1,2], 0.5) → idx=0.5*1=0.5, lo=0(1.0), hi=1(2.0), frac=0.5 → 1.5
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(percentile_inc_fn(&[arr, Value::Number(0.5)]), Value::Number(1.5));
}

#[test]
fn percentile_inc_25th() {
    // PERCENTILE.INC([1,2,3,4,5], 0.25) = 2.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentile_inc_fn(&[arr, Value::Number(0.25)]), Value::Number(2.0));
}
