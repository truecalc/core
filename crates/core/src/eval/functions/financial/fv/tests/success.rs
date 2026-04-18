use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn savings_plan() {
    // FV(0.05/12, 60, -200) — save 200/month for 5 years at 5%/year
    // ≈ 13601.22
    let args = [
        Value::Number(0.05 / 12.0),
        Value::Number(60.0),
        Value::Number(-200.0),
    ];
    assert!(approx(fv_fn(&args), 13601.22, 0.01));
}

#[test]
fn zero_rate() {
    // FV(0, 10, -100) = 1000
    let args = [Value::Number(0.0), Value::Number(10.0), Value::Number(-100.0)];
    assert!(approx(fv_fn(&args), 1000.0, 1e-9));
}

#[test]
fn with_present_value() {
    // FV(0.1, 5, 0, -1000) = 1000 * 1.1^5 ≈ 1610.51
    let args = [
        Value::Number(0.1),
        Value::Number(5.0),
        Value::Number(0.0),
        Value::Number(-1000.0),
    ];
    assert!(approx(fv_fn(&args), 1610.51, 0.01));
}
