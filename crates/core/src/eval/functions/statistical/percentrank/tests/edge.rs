use super::super::*;
use crate::types::Value;

#[test]
fn percentrank_single_element() {
    // PERCENTRANK([5], 5) = 1.0 (single element case)
    assert_eq!(
        percentrank_fn(&[Value::Number(5.0), Value::Number(5.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn percentrank_interpolation_between_values() {
    // PERCENTRANK([1,2,3,4,5], 1.5) — x between 1 and 2
    // lo_rank=0/4=0, hi_rank=1/4=0.25, frac=(1.5-1)/(2-1)=0.5 → 0+0.5*0.25=0.125
    // rounded to 3 sig = 0.125
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_fn(&[arr, Value::Number(1.5)]), Value::Number(0.125));
}

#[test]
fn percentrank_same_as_percentrank_inc() {
    // PERCENTRANK and PERCENTRANK.INC should agree
    let arr1 = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
    ]);
    let arr2 = arr1.clone();
    assert_eq!(
        percentrank_fn(&[arr1, Value::Number(2.0)]),
        crate::eval::functions::statistical::percentrank_inc::percentrank_inc_fn(&[arr2, Value::Number(2.0)])
    );
}
