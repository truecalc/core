use super::super::sqrtpi_fn;
use crate::types::Value;

#[test]
fn sqrtpi_pi() {
    // SQRTPI(PI()) = sqrt(PI*PI) = PI ≈ 3.141593
    let result = sqrtpi_fn(&[Value::Number(std::f64::consts::PI)]);
    if let Value::Number(n) = result {
        assert!((n - std::f64::consts::PI).abs() < 1e-6);
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn sqrtpi_inverse_pi() {
    // SQRTPI(1/PI()) = sqrt(1) = 1
    let result = sqrtpi_fn(&[Value::Number(1.0 / std::f64::consts::PI)]);
    if let Value::Number(n) = result {
        assert!((n - 1.0).abs() < 1e-9);
    } else {
        panic!("expected Number, got {:?}", result);
    }
}
