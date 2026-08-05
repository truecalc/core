use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn exp_large_arg_overflows() {
    // e^1000 overflows to infinity
    assert_eq!(
        exp_fn(&[Value::Number(1000.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn exp_large_negative_is_near_zero() {
    // e^(-1000) is very small but finite
    if let Value::Number(n) = exp_fn(&[Value::Number(-1000.0)]) {
        assert!(n >= 0.0);
    } else {
        panic!("Expected Number");
    }
}

#[test]
fn exp_bool_true() {
    // e^1 = e
    if let Value::Number(n) = exp_fn(&[Value::Bool(true)]) {
        assert!((n - std::f64::consts::E).abs() < 1e-10);
    } else {
        panic!("Expected Number");
    }
}
