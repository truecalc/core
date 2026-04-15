use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_no_args() {
    assert_eq!(trim_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_too_many() {
    assert_eq!(
        trim_fn(&[Value::Text("a".to_string()), Value::Text("b".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn error_propagated() {
    assert_eq!(
        trim_fn(&[Value::Error(ErrorKind::Value)]),
        Value::Error(ErrorKind::Value)
    );
}
