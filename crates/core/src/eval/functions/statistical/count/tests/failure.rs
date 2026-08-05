use super::super::*;
use crate::types::Value;

#[test]
fn count_ignores_text() {
    assert_eq!(
        count_fn(&[Value::Text("abc".to_string()), Value::Number(1.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn count_ignores_bool() {
    assert_eq!(
        count_fn(&[Value::Bool(true), Value::Bool(false), Value::Number(5.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn count_ignores_error() {
    use crate::types::ErrorKind;
    assert_eq!(
        count_fn(&[Value::Error(ErrorKind::Value), Value::Number(2.0)]),
        Value::Number(1.0)
    );
}
