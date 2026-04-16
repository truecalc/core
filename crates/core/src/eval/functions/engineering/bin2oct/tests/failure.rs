use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(bin2oct_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn invalid_char_returns_num() {
    assert_eq!(bin2oct_fn(&[Value::Text("2".to_string())]), Value::Error(ErrorKind::Num));
}

#[test]
fn places_too_small_returns_num() {
    assert_eq!(bin2oct_fn(&[Value::Text("1111111111".to_string()), Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn places_zero_returns_num() {
    assert_eq!(bin2oct_fn(&[Value::Text("1".to_string()), Value::Number(0.0)]), Value::Error(ErrorKind::Num));
}
