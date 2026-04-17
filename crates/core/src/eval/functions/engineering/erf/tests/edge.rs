use super::super::erf_fn;
use crate::types::Value;

fn approx(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() < tol }
fn unwrap(v: Value) -> f64 {
    match v { Value::Number(n) => n, other => panic!("expected Number, got {:?}", other) }
}

#[test]
fn erf_negative_input() {
    // ERF is odd: ERF(-x) = -ERF(x)
    let pos = unwrap(erf_fn(&[Value::Number(1.0)]));
    let neg = unwrap(erf_fn(&[Value::Number(-1.0)]));
    assert!(approx(neg, -pos, 1e-9));
}

#[test]
fn erf_large_input_approaches_one() {
    // ERF(6) ≈ 1.0 (error function saturates)
    let result = unwrap(erf_fn(&[Value::Number(6.0)]));
    assert!(approx(result, 1.0, 1e-6), "ERF(6) = {result}, expected ≈ 1.0");
}

#[test]
fn erf_equal_bounds_returns_zero() {
    // ERF(x, x) = ERF(x) - ERF(x) = 0
    let result = unwrap(erf_fn(&[Value::Number(1.0), Value::Number(1.0)]));
    assert!(approx(result, 0.0, 1e-10));
}
