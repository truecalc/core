use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn negative_num_chars() {
    assert_eq!(
        right_fn(&[Value::Text("Hello".to_string()), Value::Number(-1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(right_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn error_propagated() {
    assert_eq!(
        right_fn(&[Value::Error(ErrorKind::Ref)]),
        Value::Error(ErrorKind::Ref)
    );
}
