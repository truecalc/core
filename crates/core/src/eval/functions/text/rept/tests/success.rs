use super::super::*;
use crate::types::Value;

#[test]
fn basic_rept() {
    assert_eq!(
        rept_fn(&[Value::Text("ab".to_string()), Value::Number(3.0)]),
        Value::Text("ababab".to_string())
    );
}

#[test]
fn repeat_once() {
    assert_eq!(
        rept_fn(&[Value::Text("x".to_string()), Value::Number(1.0)]),
        Value::Text("x".to_string())
    );
}

#[test]
fn repeat_word() {
    assert_eq!(
        rept_fn(&[Value::Text("ha".to_string()), Value::Number(4.0)]),
        Value::Text("hahahaha".to_string())
    );
}
