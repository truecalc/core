use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn round_trip_with_pmt() {
    // PV(0.05/12, 60, -188.71) ≈ 10000
    let args = [
        Value::Number(0.05 / 12.0),
        Value::Number(60.0),
        Value::Number(-188.71),
    ];
    assert!(approx(pv_fn(&args), 10000.0, 1.0));
}

#[test]
fn zero_rate() {
    // PV(0, 10, -100) = 1000
    let args = [Value::Number(0.0), Value::Number(10.0), Value::Number(-100.0)];
    assert!(approx(pv_fn(&args), 1000.0, 1e-9));
}

#[test]
fn with_future_value() {
    // PV(0.05, 5, 0, 1000) = -1000/(1.05^5) ≈ -783.53
    let args = [
        Value::Number(0.05),
        Value::Number(5.0),
        Value::Number(0.0),
        Value::Number(1000.0),
    ];
    assert!(approx(pv_fn(&args), -783.53, 0.01));
}
