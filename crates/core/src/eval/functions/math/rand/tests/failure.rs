use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn rand_with_args_returns_value_error() {
    assert_eq!(rand_fn(&[Value::Number(1.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn randbetween_no_args_returns_value_error() {
    assert_eq!(randbetween_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn randbetween_low_greater_than_high_returns_num_error() {
    assert_eq!(
        randbetween_fn(&[Value::Number(10.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::Num)
    );
}
