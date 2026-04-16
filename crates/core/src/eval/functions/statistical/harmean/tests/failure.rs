use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn harmean_negative_value_returns_num() {
    assert_eq!(
        harmean_fn(&[Value::Number(1.0), Value::Number(-2.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn harmean_zero_value_returns_num() {
    assert_eq!(
        harmean_fn(&[Value::Number(0.0), Value::Number(4.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn harmean_no_values_returns_na() {
    assert_eq!(harmean_fn(&[]), Value::Error(ErrorKind::NA));
}
