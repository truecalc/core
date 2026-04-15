use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn round_wrong_arity() {
    assert_eq!(round_fn(&[Value::Number(1.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn roundup_wrong_arity() {
    assert_eq!(roundup_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn rounddown_non_numeric_first_arg() {
    assert_eq!(
        rounddown_fn(&[Value::Text("abc".to_string()), Value::Number(0.0)]),
        Value::Error(ErrorKind::Value)
    );
}
