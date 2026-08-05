use super::super::encodeurl_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(encodeurl_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn two_args_returns_na() {
    assert_eq!(
        encodeurl_fn(&[Value::Text("a".to_string()), Value::Text("b".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn array_arg_returns_value_error() {
    let arr = Value::Array(vec![Value::Number(1.0)]);
    assert_eq!(encodeurl_fn(&[arr]), Value::Error(ErrorKind::Value));
}
