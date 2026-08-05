use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(dec2hex_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn out_of_range_high_returns_num() {
    assert_eq!(dec2hex_fn(&[Value::Number(549_755_813_888.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn out_of_range_low_returns_num() {
    assert_eq!(dec2hex_fn(&[Value::Number(-549_755_813_889.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn places_too_small_returns_num() {
    assert_eq!(dec2hex_fn(&[Value::Number(255.0), Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn places_zero_returns_num() {
    assert_eq!(dec2hex_fn(&[Value::Number(1.0), Value::Number(0.0)]), Value::Error(ErrorKind::Num));
}
