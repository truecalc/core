use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn not_found() {
    assert_eq!(
        find_fn(&[Value::Text("x".to_string()), Value::Text("Hello".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn case_sensitive_not_found() {
    assert_eq!(
        find_fn(&[Value::Text("h".to_string()), Value::Text("Hello".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn wrong_arity() {
    assert_eq!(
        find_fn(&[Value::Text("x".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}
