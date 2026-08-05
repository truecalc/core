use super::super::encodeurl_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn empty_string_returns_empty_string() {
    assert_eq!(
        encodeurl_fn(&[Value::Text(String::new())]),
        Value::Text(String::new())
    );
}

#[test]
fn wrong_arg_count_zero_returns_error() {
    assert_eq!(encodeurl_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arg_count_two_returns_error() {
    assert_eq!(
        encodeurl_fn(&[
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
        ]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn array_arg_returns_value_error() {
    assert_eq!(
        encodeurl_fn(&[Value::Array(vec![Value::Number(1.0)])]),
        Value::Error(ErrorKind::Value)
    );
}
