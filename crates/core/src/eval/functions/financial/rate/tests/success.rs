use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn monthly_rate_for_loan() {
    // RATE(60, -188.71, 10000) ≈ 0.05/12 ≈ 0.004167
    let args = [
        Value::Number(60.0),
        Value::Number(-188.71),
        Value::Number(10000.0),
    ];
    assert!(approx(rate_fn(&args), 0.05 / 12.0, 0.0001));
}

#[test]
fn annual_rate_simple() {
    // RATE(5, -263.80, 1000) should be ≈ 10%
    // PMT(0.1, 5, 1000) ≈ -263.80
    let args = [
        Value::Number(5.0),
        Value::Number(-263.80),
        Value::Number(1000.0),
    ];
    assert!(approx(rate_fn(&args), 0.1, 0.001));
}

#[test]
fn rate_with_fv() {
    // Round-trip: compute PMT then recover RATE
    // PMT(0.05, 10, 1000, 0) ≈ -129.50
    // RATE(10, -129.50, 1000) ≈ 0.05
    let args = [
        Value::Number(10.0),
        Value::Number(-129.50),
        Value::Number(1000.0),
    ];
    assert!(approx(rate_fn(&args), 0.05, 0.001));
}
