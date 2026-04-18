use super::super::*;
use crate::types::Value;

#[test]
fn shift_by_zero() {
    assert_eq!(bitlshift_fn(&[Value::Number(4.0), Value::Number(0.0)]), Value::Number(4.0));
}

#[test]
fn shift_left_by_one() {
    assert_eq!(bitlshift_fn(&[Value::Number(4.0), Value::Number(1.0)]), Value::Number(8.0));
}

#[test]
fn shift_left_by_two() {
    assert_eq!(bitlshift_fn(&[Value::Number(1.0), Value::Number(2.0)]), Value::Number(4.0));
}

#[test]
fn negative_shift_acts_as_right_shift() {
    assert_eq!(bitlshift_fn(&[Value::Number(8.0), Value::Number(-1.0)]), Value::Number(4.0));
}

#[test]
fn zero_number() {
    assert_eq!(bitlshift_fn(&[Value::Number(0.0), Value::Number(4.0)]), Value::Number(0.0));
}
