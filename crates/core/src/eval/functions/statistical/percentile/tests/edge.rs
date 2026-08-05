use super::super::*;
use crate::types::Value;

#[test]
fn percentile_single_value() {
    // PERCENTILE([7], 0.5) = 7.0 (only one element)
    assert_eq!(percentile_fn(&[Value::Number(7.0), Value::Number(0.5)]), Value::Number(7.0));
}

#[test]
fn percentile_interpolation() {
    // PERCENTILE([1,2,3,4,5], 0.3) → idx = 0.3*4 = 1.2, lo=1(2.0), hi=2(3.0), frac=0.2
    // = 2.0 * 0.8 + 3.0 * 0.2 = 1.6 + 0.6 = 2.2
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentile_fn(&[arr, Value::Number(0.3)]), Value::Number(2.2));
}

#[test]
fn percentile_same_as_percentile_inc() {
    // PERCENTILE and PERCENTILE.INC should agree
    let arr1 = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
    ]);
    let arr2 = arr1.clone();
    assert_eq!(
        percentile_fn(&[arr1, Value::Number(0.5)]),
        crate::eval::functions::statistical::percentile_inc::percentile_inc_fn(&[arr2, Value::Number(0.5)])
    );
}
