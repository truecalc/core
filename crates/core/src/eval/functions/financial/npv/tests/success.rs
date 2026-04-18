use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn basic_npv() {
    // NPV(0.1, -1000, 200, 300, 400, 500)
    // = -1000/1.1 + 200/1.1^2 + 300/1.1^3 + 400/1.1^4 + 500/1.1^5
    // ≈ -909.09 + 165.29 + 225.39 + 273.21 + 310.46 ≈ 65.26
    let args = [
        Value::Number(0.1),
        Value::Number(-1000.0),
        Value::Number(200.0),
        Value::Number(300.0),
        Value::Number(400.0),
        Value::Number(500.0),
    ];
    assert!(approx(npv_fn(&args), 65.26, 0.1));
}

#[test]
fn zero_rate_sums_cash_flows() {
    // At rate=0, NPV = sum of all cash flows
    let args = [
        Value::Number(0.0),
        Value::Number(-100.0),
        Value::Number(50.0),
        Value::Number(80.0),
    ];
    assert!(approx(npv_fn(&args), 30.0, 1e-9));
}

#[test]
fn single_cash_flow() {
    // NPV(0.1, 110) = 110/1.1 = 100
    let args = [Value::Number(0.1), Value::Number(110.0)];
    assert!(approx(npv_fn(&args), 100.0, 1e-9));
}
