use super::super::*;
use crate::types::{ErrorKind, Value};

fn run(n: f64) -> Value {
    roman_fn(&[Value::Number(n)])
}

#[test]
fn zero_returns_value_error() {
    assert_eq!(run(0.0), Value::Error(ErrorKind::Value));
}

#[test]
fn negative_returns_value_error() {
    assert_eq!(run(-1.0), Value::Error(ErrorKind::Value));
}

#[test]
fn no_args_returns_na() {
    assert_eq!(roman_fn(&[]), Value::Error(ErrorKind::NA));
}
