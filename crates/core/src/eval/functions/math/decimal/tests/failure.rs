use super::super::decimal_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(decimal_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_one_arg() {
    assert_eq!(
        decimal_fn(&[Value::Text("FF".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn invalid_char_for_base() {
    // DECIMAL("G", 16) → #NUM! ("G" is not a valid hex digit)
    assert_eq!(
        decimal_fn(&[Value::Text("G".to_string()), Value::Number(16.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn base_too_small() {
    // DECIMAL("1", 1) → #NUM!
    assert_eq!(
        decimal_fn(&[Value::Text("1".to_string()), Value::Number(1.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn base_too_large() {
    // DECIMAL("1", 37) → #NUM!
    assert_eq!(
        decimal_fn(&[Value::Text("1".to_string()), Value::Number(37.0)]),
        Value::Error(ErrorKind::Num)
    );
}
