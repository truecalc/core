use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn type1_vs_type0() {
    let args0 = [Value::Number(0.05), Value::Number(5.0), Value::Number(-100.0), Value::Number(0.0), Value::Number(0.0)];
    let args1 = [Value::Number(0.05), Value::Number(5.0), Value::Number(-100.0), Value::Number(0.0), Value::Number(1.0)];
    let fv0 = fv_fn(&args0);
    let fv1 = fv_fn(&args1);
    assert!(matches!(fv0, Value::Number(_)));
    assert!(matches!(fv1, Value::Number(_)));
    assert_ne!(fv0, fv1);
}

#[test]
fn pv_defaults_to_zero() {
    let args3 = [Value::Number(0.05), Value::Number(10.0), Value::Number(-100.0)];
    let args4 = [Value::Number(0.05), Value::Number(10.0), Value::Number(-100.0), Value::Number(0.0)];
    assert_eq!(fv_fn(&args3), fv_fn(&args4));
}

#[test]
fn zero_rate_with_pv() {
    // FV(0, 5, -100, -200) = 200 + 500 = 700
    let args = [
        Value::Number(0.0),
        Value::Number(5.0),
        Value::Number(-100.0),
        Value::Number(-200.0),
    ];
    assert!(approx(fv_fn(&args), 700.0, 1e-9));
}
