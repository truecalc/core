use super::super::*;
use crate::types::Value;

#[test]
fn abs_large_negative() {
    assert_eq!(abs_fn(&[Value::Number(-1e308)]), Value::Number(1e308));
}

#[test]
fn abs_bool_true() {
    assert_eq!(abs_fn(&[Value::Bool(true)]), Value::Number(1.0));
}

#[test]
fn abs_numeric_text() {
    assert_eq!(
        abs_fn(&[Value::Text("-3.5".to_string())]),
        Value::Number(3.5)
    );
}
