use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn start_beyond_end() {
    assert_eq!(
        find_fn(&[Value::Text("l".to_string()), Value::Text("Hello".to_string()), Value::Number(10.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn find_empty_string() {
    assert_eq!(
        find_fn(&[Value::Text("".to_string()), Value::Text("Hello".to_string())]),
        Value::Number(1.0)
    );
}

#[test]
fn start_zero_is_error() {
    assert_eq!(
        find_fn(&[Value::Text("l".to_string()), Value::Text("Hello".to_string()), Value::Number(0.0)]),
        Value::Error(ErrorKind::Value)
    );
}
