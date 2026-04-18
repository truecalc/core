use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn custom_guess() {
    // Provide a guess closer to the answer for faster convergence
    let args = [
        Value::Number(60.0),
        Value::Number(-188.71),
        Value::Number(10000.0),
        Value::Number(0.0),
        Value::Number(0.0),
        Value::Number(0.004), // guess near 0.05/12
    ];
    assert!(approx(rate_fn(&args), 0.05 / 12.0, 0.0001));
}

#[test]
fn type1_rate() {
    // type=1 (beginning of period) should still converge
    let args = [
        Value::Number(5.0),
        Value::Number(-250.0),
        Value::Number(1000.0),
        Value::Number(0.0),
        Value::Number(1.0),
    ];
    let result = rate_fn(&args);
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn fv_non_zero() {
    // RATE with non-zero fv should still converge
    let args = [
        Value::Number(10.0),
        Value::Number(-100.0),
        Value::Number(500.0),
        Value::Number(200.0),
    ];
    let result = rate_fn(&args);
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn guess_zero_forces_r0_branch() {
    // RATE with guess=0 forces the r=0 branch on first iteration
    // RATE(60, -188.71, 10000, 0, 0, 0.0) should converge to ≈ 0.05/12 ≈ 0.004167
    let args = [
        Value::Number(60.0),
        Value::Number(-188.71),
        Value::Number(10000.0),
        Value::Number(0.0),
        Value::Number(0.0),
        Value::Number(0.0),
    ];
    assert!(approx(rate_fn(&args), 0.05 / 12.0, 1e-4));
}
