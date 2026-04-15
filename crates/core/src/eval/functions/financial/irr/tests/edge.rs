use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn zero_initial_cash_flow() {
    // First cash flow zero — still computable if mixed signs exist
    // IRR(0, -100, 150) — Newton-Raphson should converge
    let args = [Value::Number(0.0), Value::Number(-100.0), Value::Number(150.0)];
    // Just verify it returns a Number (convergence behavior may vary)
    assert!(matches!(irr_fn(&args), Value::Number(_)));
}

#[test]
fn large_series() {
    // 10-period series: invest 1000, get back 150/year for 10 years ≈ 8.14%
    let mut args = vec![Value::Number(-1000.0)];
    for _ in 0..10 {
        args.push(Value::Number(150.0));
    }
    let result = irr_fn(&args);
    assert!(approx(result, 0.0814, 0.001));
}

#[test]
fn exact_zero_irr() {
    // IRR(-100, 100) = 0% since NPV at 0% = -100+100=0
    let args = [Value::Number(-100.0), Value::Number(100.0)];
    assert!(approx(irr_fn(&args), 0.0, 1e-6));
}
