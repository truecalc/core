use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(dec2oct_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn out_of_range_high_returns_num() {
    assert_eq!(dec2oct_fn(&[Value::Number(536_870_912.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn out_of_range_low_returns_num() {
    assert_eq!(dec2oct_fn(&[Value::Number(-536_870_913.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn places_too_small_returns_num() {
    assert_eq!(dec2oct_fn(&[Value::Number(536_870_911.0), Value::Number(2.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn places_zero_returns_num() {
    assert_eq!(dec2oct_fn(&[Value::Number(1.0), Value::Number(0.0)]), Value::Error(ErrorKind::Num));
}
