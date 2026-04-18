use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn power_negative_base_integer_exp() {
    assert_eq!(
        power_fn(&[Value::Number(-2.0), Value::Number(3.0)]),
        Value::Number(-8.0)
    );
}

#[test]
fn power_fractional_exponent() {
    // 4^0.5 = 2
    assert_eq!(
        power_fn(&[Value::Number(4.0), Value::Number(0.5)]),
        Value::Number(2.0)
    );
}

#[test]
fn power_overflow_returns_num_error() {
    assert_eq!(
        power_fn(&[Value::Number(f64::MAX), Value::Number(2.0)]),
        Value::Error(ErrorKind::Num)
    );
}
