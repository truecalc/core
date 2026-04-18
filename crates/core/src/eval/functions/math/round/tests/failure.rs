use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn round_no_args_returns_na() {
    // 0 args → #N/A; 1 arg is now valid (num_digits defaults to 0)
    assert_eq!(round_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn roundup_wrong_arity() {
    assert_eq!(roundup_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn rounddown_non_numeric_first_arg() {
    assert_eq!(
        rounddown_fn(&[Value::Text("abc".to_string()), Value::Number(0.0)]),
        Value::Error(ErrorKind::Value)
    );
}
