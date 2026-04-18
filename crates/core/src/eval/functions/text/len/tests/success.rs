use super::super::*;
use crate::types::Value;

#[test]
fn basic_len() {
    assert_eq!(
        len_fn(&[Value::Text("Hello".to_string())]),
        Value::Number(5.0)
    );
}

#[test]
fn empty_string() {
    assert_eq!(
        len_fn(&[Value::Text("".to_string())]),
        Value::Number(0.0)
    );
}

#[test]
fn number_coerced() {
    assert_eq!(
        len_fn(&[Value::Number(123.0)]),
        Value::Number(3.0)
    );
}
