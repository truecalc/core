use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn unparseable_text() {
    assert_eq!(
        value_fn(&[Value::Text("hello".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn wrong_arity_no_args() {
    assert_eq!(value_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn error_propagated() {
    assert_eq!(
        value_fn(&[Value::Error(ErrorKind::NA)]),
        Value::Error(ErrorKind::NA)
    );
}
