use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn sqrt_large_number() {
    assert_eq!(sqrt_fn(&[Value::Number(1e8)]), Value::Number(1e4));
}

#[test]
fn sqrt_one() {
    assert_eq!(sqrt_fn(&[Value::Number(1.0)]), Value::Number(1.0));
}

#[test]
fn sqrt_large_negative_returns_num_error() {
    assert_eq!(
        sqrt_fn(&[Value::Number(-1e100)]),
        Value::Error(ErrorKind::Num)
    );
}
