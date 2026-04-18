use super::super::not_fn;
use crate::types::Value;

#[test]
fn not_true_returns_false() {
    assert_eq!(not_fn(&[Value::Bool(true)]), Value::Bool(false));
}

#[test]
fn not_false_returns_true() {
    assert_eq!(not_fn(&[Value::Bool(false)]), Value::Bool(true));
}

#[test]
fn not_nonzero_returns_false() {
    assert_eq!(not_fn(&[Value::Number(1.0)]), Value::Bool(false));
}
