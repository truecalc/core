use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn geomean_negative_value_returns_num() {
    assert_eq!(
        geomean_fn(&[Value::Number(4.0), Value::Number(-1.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn geomean_zero_value_returns_num() {
    assert_eq!(
        geomean_fn(&[Value::Number(0.0), Value::Number(4.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn geomean_no_values_returns_num() {
    assert_eq!(geomean_fn(&[]), Value::Error(ErrorKind::Num));
}
