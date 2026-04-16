use super::maxa_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn plain_numbers() {
    assert_eq!(
        maxa_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn true_treated_as_1() {
    assert_eq!(
        maxa_fn(&[Value::Bool(true), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn false_and_zero() {
    assert_eq!(
        maxa_fn(&[Value::Bool(false), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn text_direct_arg_returns_value_error() {
    assert_eq!(
        maxa_fn(&[Value::Text("text".to_string()), Value::Number(5.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn no_args_returns_na() {
    assert_eq!(maxa_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn all_empty_returns_na() {
    assert_eq!(maxa_fn(&[Value::Empty]), Value::Error(ErrorKind::NA));
}
