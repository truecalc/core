use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn fvschedule_empty_rates_returns_principal() {
    // FVSCHEDULE(1000, []) = 1000 (no compounding)
    let args = [Value::Number(1000.0), Value::Array(vec![])];
    assert!(approx(fvschedule_fn(&args), 1000.0, 1e-9));
}

#[test]
fn sln_equal_cost_and_salvage_returns_zero() {
    // SLN(5000, 5000, 5) = 0 (no depreciation)
    let args = [Value::Number(5000.0), Value::Number(5000.0), Value::Number(5.0)];
    assert!(approx(sln_fn(&args), 0.0, 1e-9));
}

#[test]
fn xnpv_zero_rate_sums_cashflows() {
    // At rate=0, XNPV = sum of all cashflows = -1000 + 1100 = 100
    let args = [
        Value::Number(0.0),
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1100.0)]),
        Value::Array(vec![Value::Number(44927.0), Value::Number(45292.0)]),
    ];
    assert!(approx(xnpv_fn(&args), 100.0, 1e-6));
}
