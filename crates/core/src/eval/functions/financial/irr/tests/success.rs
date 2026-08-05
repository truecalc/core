use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn basic_irr() {
    // IRR(-1000, 200, 300, 400, 500) ≈ 0.12826 (12.83%)
    let args = [
        Value::Number(-1000.0),
        Value::Number(200.0),
        Value::Number(300.0),
        Value::Number(400.0),
        Value::Number(500.0),
    ];
    assert!(approx(irr_fn(&args), 0.12826, 0.001));
}

#[test]
fn simple_irr_known_result() {
    // IRR(-100, 110) = 10%
    let args = [Value::Number(-100.0), Value::Number(110.0)];
    assert!(approx(irr_fn(&args), 0.1, 1e-6));
}

#[test]
fn irr_two_periods() {
    // IRR(-100, 0, 121) = 10% (sqrt(1.21)-1 = 0.1)
    let args = [Value::Number(-100.0), Value::Number(0.0), Value::Number(121.0)];
    assert!(approx(irr_fn(&args), 0.1, 1e-6));
}
