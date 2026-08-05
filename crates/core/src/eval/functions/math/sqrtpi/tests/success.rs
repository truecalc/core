use super::super::sqrtpi_fn;
use crate::types::Value;

#[test]
fn sqrtpi_one() {
    // SQRTPI(1) = sqrt(PI) ≈ 1.7724538509
    let result = sqrtpi_fn(&[Value::Number(1.0)]);
    if let Value::Number(n) = result {
        assert!((n - 1.7724538509_f64).abs() < 1e-9);
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn sqrtpi_four() {
    // SQRTPI(4) = 2*sqrt(PI) ≈ 3.544908
    let result = sqrtpi_fn(&[Value::Number(4.0)]);
    if let Value::Number(n) = result {
        assert!((n - 3.5449077_f64).abs() < 1e-6);
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn sqrtpi_zero() {
    // SQRTPI(0) = 0
    assert_eq!(sqrtpi_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}
