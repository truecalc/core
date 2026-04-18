use super::super::*;
use crate::types::Value;

#[test]
fn vara_basic_numbers() {
    // [2, 4, 6]: mean=4, sample var=((2-4)²+(4-4)²+(6-4)²)/2=8/2=4
    let result = vara_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn vara_two_values() {
    // [1, 3]: mean=2, var=((1-2)²+(3-2)²)/1=2
    let result = vara_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn vara_numbers_only() {
    // Same as VAR.S when only numbers present
    let result = vara_fn(&[Value::Number(0.0), Value::Number(10.0)]);
    assert_eq!(result, Value::Number(50.0));
}
