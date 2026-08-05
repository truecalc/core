use super::super::*;
use crate::types::Value;

#[test]
fn cos_of_pi() {
    if let Value::Number(n) = cos_fn(&[Value::Number(std::f64::consts::PI)]) {
        assert!((n - (-1.0)).abs() < 1e-10);
    } else {
        panic!("Expected Number");
    }
}

#[test]
fn sin_of_pi_is_near_zero() {
    if let Value::Number(n) = sin_fn(&[Value::Number(std::f64::consts::PI)]) {
        assert!(n.abs() < 1e-10);
    } else {
        panic!("Expected Number");
    }
}

#[test]
fn tan_of_pi_over_4_is_near_one() {
    if let Value::Number(n) = tan_fn(&[Value::Number(std::f64::consts::FRAC_PI_4)]) {
        assert!((n - 1.0).abs() < 1e-10);
    } else {
        panic!("Expected Number");
    }
}
