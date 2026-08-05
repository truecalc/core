use super::super::isurl_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn plain_text_returns_false() {
    assert_eq!(
        isurl_fn(&[Value::Text("notaurl".to_string())]),
        Value::Bool(false)
    );
}

#[test]
fn empty_string_returns_false() {
    assert_eq!(
        isurl_fn(&[Value::Text(String::new())]),
        Value::Bool(false)
    );
}

#[test]
fn number_returns_false() {
    assert_eq!(
        isurl_fn(&[Value::Number(42.0)]),
        Value::Bool(false)
    );
}

#[test]
fn wrong_arg_count_zero_returns_error() {
    assert_eq!(isurl_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arg_count_two_returns_error() {
    assert_eq!(
        isurl_fn(&[
            Value::Text("https://a.com".to_string()),
            Value::Text("extra".to_string()),
        ]),
        Value::Error(ErrorKind::NA)
    );
}
