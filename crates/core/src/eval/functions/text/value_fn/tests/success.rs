use super::super::*;
use crate::types::Value;

#[test]
fn parse_integer() {
    assert_eq!(
        value_fn(&[Value::Text("42".to_string())]),
        Value::Number(42.0)
    );
}

#[test]
fn parse_float() {
    assert_eq!(
        value_fn(&[Value::Text("3.14".to_string())]),
        Value::Number(3.14)
    );
}

#[test]
fn parse_negative() {
    assert_eq!(
        value_fn(&[Value::Text("-5.5".to_string())]),
        Value::Number(-5.5)
    );
}
