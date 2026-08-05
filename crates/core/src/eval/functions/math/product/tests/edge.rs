use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn product_with_zero() {
    assert_eq!(
        product_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn product_with_negatives() {
    assert_eq!(
        product_fn(&[Value::Number(-2.0), Value::Number(-3.0)]),
        Value::Number(6.0)
    );
}

#[test]
fn overflow_returns_num_error() {
    assert_eq!(
        product_fn(&[Value::Number(f64::MAX), Value::Number(2.0)]),
        Value::Error(ErrorKind::Num)
    );
}
