use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_too_few() {
    assert_eq!(
        substitute_fn(&[Value::Text("a".to_string()), Value::Text("b".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn instance_zero_is_error() {
    assert_eq!(
        substitute_fn(&[
            Value::Text("aaa".to_string()),
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
            Value::Number(0.0),
        ]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn error_propagated() {
    assert_eq!(
        substitute_fn(&[
            Value::Error(ErrorKind::Ref),
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
        ]),
        Value::Error(ErrorKind::Ref)
    );
}
