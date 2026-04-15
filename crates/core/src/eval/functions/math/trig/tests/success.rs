use super::super::*;
use crate::types::Value;

#[test]
fn pi_returns_pi() {
    assert_eq!(pi_fn(&[]), Value::Number(std::f64::consts::PI));
}

#[test]
fn sin_of_zero() {
    assert_eq!(sin_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn cos_of_zero() {
    assert_eq!(cos_fn(&[Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn tan_of_zero() {
    assert_eq!(tan_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn sin_of_pi_over_2() {
    if let Value::Number(n) = sin_fn(&[Value::Number(std::f64::consts::FRAC_PI_2)]) {
        assert!((n - 1.0).abs() < 1e-10);
    } else {
        panic!("Expected Number");
    }
}
