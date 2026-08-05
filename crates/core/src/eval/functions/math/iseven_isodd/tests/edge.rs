use super::super::*;
use crate::types::Value;

#[test]
fn iseven_negative_odd() {
    assert_eq!(iseven_fn(&[Value::Number(-3.0)]), Value::Bool(false));
}

#[test]
fn isodd_zero_is_not_odd() {
    assert_eq!(isodd_fn(&[Value::Number(0.0)]), Value::Bool(false));
}

#[test]
fn isodd_negative_even() {
    assert_eq!(isodd_fn(&[Value::Number(-2.0)]), Value::Bool(false));
}

#[test]
fn iseven_fractional_truncates_to_odd() {
    // 3.9 truncates to 3, which is odd → ISEVEN = false
    assert_eq!(iseven_fn(&[Value::Number(3.9)]), Value::Bool(false));
}
