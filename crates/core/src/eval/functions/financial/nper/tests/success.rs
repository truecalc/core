use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn nper_for_loan() {
    // NPER(0.05/12, -188.71, 10000) ≈ 60
    let args = [
        Value::Number(0.05 / 12.0),
        Value::Number(-188.71),
        Value::Number(10000.0),
    ];
    assert!(approx(nper_fn(&args), 60.0, 0.01));
}

#[test]
fn zero_rate() {
    // NPER(0, -100, 1000) = 10
    let args = [Value::Number(0.0), Value::Number(-100.0), Value::Number(1000.0)];
    assert!(approx(nper_fn(&args), 10.0, 1e-9));
}

#[test]
fn nper_with_fv() {
    // NPER(0.1, -100, 500, 0) — should return a positive number
    let args = [
        Value::Number(0.1),
        Value::Number(-100.0),
        Value::Number(500.0),
        Value::Number(0.0),
    ];
    let result = nper_fn(&args);
    assert!(matches!(result, Value::Number(n) if n > 0.0));
}
