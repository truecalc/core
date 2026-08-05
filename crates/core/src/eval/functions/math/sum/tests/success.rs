use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn sum_of_numbers() {
    assert_eq!(
        sum_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(6.0)
    );
}

#[test]
fn single_number() {
    assert_eq!(sum_fn(&[Value::Number(5.0)]), Value::Number(5.0));
}

#[test]
fn single_zero_value() {
    assert_eq!(sum_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn bool_as_zero_or_one() {
    assert_eq!(
        sum_fn(&[Value::Bool(true), Value::Bool(false), Value::Bool(true)]),
        Value::Number(2.0)
    );
}

#[test]
fn numeric_text() {
    assert_eq!(
        sum_fn(&[Value::Text("5".to_string()), Value::Number(3.0)]),
        Value::Number(8.0)
    );
}
