use super::super::*;
use crate::types::Value;

#[test]
fn percentrank_exc_interpolation_between_values() {
    // PERCENTRANK.EXC([1,2,3,4,5], 1.5): x between 1 and 2
    // count_below=1 values < 1.5 (just 1.0), count_equal=0
    // lo_rank=1/6, hi_rank=2/6, lo_val=1.0, hi_val=2.0, frac=0.5
    // result = 1/6 + 0.5*(2/6-1/6) = 1/6 + 0.5/6 = 1.5/6 = 0.25
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_exc_fn(&[arr, Value::Number(1.5)]), Value::Number(0.25));
}

#[test]
fn percentrank_exc_two_element_dataset() {
    // PERCENTRANK.EXC([1,3], 2): between 1 and 3
    // lo_rank=1/3, hi_rank=2/3, frac=(2-1)/(3-1)=0.5
    // result=1/3+0.5*(2/3-1/3)=1/3+1/6=0.5
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(3.0)]);
    assert_eq!(percentrank_exc_fn(&[arr, Value::Number(2.0)]), Value::Number(0.5));
}
