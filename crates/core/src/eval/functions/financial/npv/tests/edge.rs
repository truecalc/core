use super::super::*;
use crate::types::{ErrorKind, Value};

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn negative_rate() {
    // Negative discount rate is valid math — should return a Number
    let args = [Value::Number(-0.05), Value::Number(100.0), Value::Number(100.0)];
    assert!(matches!(npv_fn(&args), Value::Number(_)));
}

#[test]
fn rate_minus_one_returns_num() {
    // rate = -1.0 causes division by zero → #NUM!
    let args = [Value::Number(-1.0), Value::Number(100.0)];
    assert_eq!(npv_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn all_positive_cash_flows() {
    let args = [Value::Number(0.05), Value::Number(100.0), Value::Number(200.0)];
    let result = npv_fn(&args);
    assert!(approx(result, 100.0 / 1.05 + 200.0 / 1.05_f64.powi(2), 1e-9));
}
