use super::super::base_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(base_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_one_arg() {
    assert_eq!(base_fn(&[Value::Number(10.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn negative_value() {
    // BASE(-1, 2) → #NUM!
    assert_eq!(
        base_fn(&[Value::Number(-1.0), Value::Number(2.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn base_too_small() {
    // BASE(10, 1) → #NUM!
    assert_eq!(
        base_fn(&[Value::Number(10.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn base_too_large() {
    // BASE(10, 37) → #NUM!
    assert_eq!(
        base_fn(&[Value::Number(10.0), Value::Number(37.0)]),
        Value::Error(ErrorKind::Num)
    );
}
