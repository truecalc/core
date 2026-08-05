use super::super::*;
use crate::types::Value;

#[test]
fn greater_than_step_returns_one() {
    assert_eq!(gestep_fn(&[Value::Number(5.0), Value::Number(4.0)]), Value::Number(1.0));
}

#[test]
fn equal_to_step_returns_one() {
    assert_eq!(gestep_fn(&[Value::Number(5.0), Value::Number(5.0)]), Value::Number(1.0));
}

#[test]
fn less_than_step_returns_zero() {
    assert_eq!(gestep_fn(&[Value::Number(3.0), Value::Number(5.0)]), Value::Number(0.0));
}

#[test]
fn default_step_zero_positive_returns_one() {
    assert_eq!(gestep_fn(&[Value::Number(1.0)]), Value::Number(1.0));
}

#[test]
fn default_step_zero_equal_returns_one() {
    assert_eq!(gestep_fn(&[Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn default_step_zero_negative_returns_zero() {
    assert_eq!(gestep_fn(&[Value::Number(-1.0)]), Value::Number(0.0));
}

#[test]
fn negative_numbers() {
    assert_eq!(gestep_fn(&[Value::Number(-2.0), Value::Number(-3.0)]), Value::Number(1.0));
}
