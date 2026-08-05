use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn negative_times() {
    assert_eq!(
        rept_fn(&[Value::Text("x".to_string()), Value::Number(-1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn wrong_arity_no_args() {
    assert_eq!(rept_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn error_propagated() {
    assert_eq!(
        rept_fn(&[Value::Error(ErrorKind::Ref), Value::Number(2.0)]),
        Value::Error(ErrorKind::Ref)
    );
}
