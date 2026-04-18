use super::super::*;
use crate::types::Value;

#[test]
fn sign_positive() {
    assert_eq!(sign_fn(&[Value::Number(5.0)]), Value::Number(1.0));
}

#[test]
fn sign_negative() {
    assert_eq!(sign_fn(&[Value::Number(-3.0)]), Value::Number(-1.0));
}

#[test]
fn sign_zero() {
    assert_eq!(sign_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}
