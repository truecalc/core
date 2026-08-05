use super::super::*;
use crate::types::Value;

#[test]
fn equal_values_returns_one() {
    assert_eq!(delta_fn(&[Value::Number(5.0), Value::Number(5.0)]), Value::Number(1.0));
}

#[test]
fn unequal_values_returns_zero() {
    assert_eq!(delta_fn(&[Value::Number(5.0), Value::Number(4.0)]), Value::Number(0.0));
}

#[test]
fn default_second_arg_zero_equal() {
    assert_eq!(delta_fn(&[Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn default_second_arg_zero_unequal() {
    assert_eq!(delta_fn(&[Value::Number(1.0)]), Value::Number(0.0));
}

#[test]
fn both_zero_returns_one() {
    assert_eq!(delta_fn(&[Value::Number(0.0), Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn negative_equal_returns_one() {
    assert_eq!(delta_fn(&[Value::Number(-3.0), Value::Number(-3.0)]), Value::Number(1.0));
}
