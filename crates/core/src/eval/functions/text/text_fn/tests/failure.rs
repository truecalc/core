use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_no_args() {
    assert_eq!(text_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_one_arg() {
    assert_eq!(
        text_fn(&[Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn non_number_text_arg() {
    assert_eq!(
        text_fn(&[Value::Text("hello".to_string()), Value::Text("0".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}
