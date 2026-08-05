use super::super::*;
use crate::types::Value;

#[test]
fn sign_very_small_positive() {
    assert_eq!(sign_fn(&[Value::Number(1e-300)]), Value::Number(1.0));
}

#[test]
fn sign_very_small_negative() {
    assert_eq!(sign_fn(&[Value::Number(-1e-300)]), Value::Number(-1.0));
}

#[test]
fn sign_bool_false_is_zero() {
    assert_eq!(sign_fn(&[Value::Bool(false)]), Value::Number(0.0));
}
