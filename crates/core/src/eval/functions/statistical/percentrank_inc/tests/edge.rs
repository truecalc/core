use super::super::*;
use crate::types::Value;

#[test]
fn percentrank_inc_single_element() {
    // PERCENTRANK.INC([5], 5) = 1.0
    assert_eq!(
        percentrank_inc_fn(&[Value::Number(5.0), Value::Number(5.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn percentrank_inc_interpolation_between_values() {
    // PERCENTRANK.INC([1,2,3,4,5], 1.5): x=1.5 between 1 and 2
    // count_below=1, count_equal=0
    // lo_rank=(1-1)/4=0/4=0.0, hi_rank=1/4=0.25
    // lo_val=1.0, hi_val=2.0, frac=0.5
    // result=0.0+0.5*0.25=0.125
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_inc_fn(&[arr, Value::Number(1.5)]), Value::Number(0.125));
}

#[test]
fn percentrank_inc_default_sig_3_digits() {
    // PERCENTRANK.INC([1,2,3,4,5,6], 2) = 1/5=0.2
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0), Value::Number(6.0),
    ]);
    assert_eq!(percentrank_inc_fn(&[arr, Value::Number(2.0)]), Value::Number(0.2));
}
