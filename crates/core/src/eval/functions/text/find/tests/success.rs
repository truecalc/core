use super::super::*;
use crate::types::Value;

#[test]
fn basic_find() {
    assert_eq!(
        find_fn(&[Value::Text("l".to_string()), Value::Text("Hello".to_string())]),
        Value::Number(3.0)
    );
}

#[test]
fn find_with_start() {
    assert_eq!(
        find_fn(&[Value::Text("l".to_string()), Value::Text("Hello".to_string()), Value::Number(4.0)]),
        Value::Number(4.0)
    );
}

#[test]
fn find_at_beginning() {
    assert_eq!(
        find_fn(&[Value::Text("H".to_string()), Value::Text("Hello".to_string())]),
        Value::Number(1.0)
    );
}
