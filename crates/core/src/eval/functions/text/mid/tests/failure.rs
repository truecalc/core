use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn start_less_than_one() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(0.0), Value::Number(3.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn negative_num_chars() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(1.0), Value::Number(-1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn wrong_arity() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}
