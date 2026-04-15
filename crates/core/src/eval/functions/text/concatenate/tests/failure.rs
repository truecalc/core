use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_no_args() {
    assert_eq!(concatenate_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn error_propagated() {
    assert_eq!(
        concatenate_fn(&[Value::Text("a".to_string()), Value::Error(ErrorKind::Ref)]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn array_causes_error() {
    assert_eq!(
        concatenate_fn(&[Value::Array(vec![Value::Number(1.0)])]),
        Value::Error(ErrorKind::Value)
    );
}
