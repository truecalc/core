use super::super::*;
use crate::types::Value;

#[test]
fn exp_zero_is_one() {
    assert_eq!(exp_fn(&[Value::Number(0.0)]), Value::Number(1.0));
}

#[test]
fn exp_one_is_e() {
    if let Value::Number(n) = exp_fn(&[Value::Number(1.0)]) {
        assert!((n - std::f64::consts::E).abs() < 1e-10);
    } else {
        panic!("Expected Number");
    }
}

#[test]
fn exp_negative_arg() {
    // e^(-1) ≈ 0.3679
    if let Value::Number(n) = exp_fn(&[Value::Number(-1.0)]) {
        assert!((n - 1.0 / std::f64::consts::E).abs() < 1e-10);
    } else {
        panic!("Expected Number");
    }
}
