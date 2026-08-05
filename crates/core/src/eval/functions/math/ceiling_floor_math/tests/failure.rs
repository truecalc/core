use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn ceiling_math_no_args() {
    assert_eq!(ceiling_math_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn ceiling_math_non_numeric() {
    assert_eq!(
        ceiling_math_fn(&[Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn ceiling_precise_no_args() {
    assert_eq!(ceiling_precise_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn ceiling_precise_too_many_args() {
    assert_eq!(
        ceiling_precise_fn(&[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn floor_math_no_args() {
    assert_eq!(floor_math_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn floor_math_non_numeric() {
    assert_eq!(
        floor_math_fn(&[Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn floor_precise_no_args() {
    assert_eq!(floor_precise_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn iso_ceiling_no_args() {
    assert_eq!(iso_ceiling_fn(&[]), Value::Error(ErrorKind::NA));
}
