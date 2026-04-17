use super::super::isurl_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(isurl_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn two_args_returns_na() {
    assert_eq!(
        isurl_fn(&[Value::Text("a".to_string()), Value::Text("b".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}
