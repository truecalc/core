use super::super::*;
use crate::types::Value;

#[test]
fn sqrt_perfect_square() {
    assert_eq!(sqrt_fn(&[Value::Number(9.0)]), Value::Number(3.0));
}

#[test]
fn sqrt_of_four() {
    assert_eq!(sqrt_fn(&[Value::Number(4.0)]), Value::Number(2.0));
}

#[test]
fn sqrt_of_zero() {
    assert_eq!(sqrt_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}
