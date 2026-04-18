use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn type1_beginning_of_period() {
    // With type=1, PV should differ from type=0
    let args0 = [Value::Number(0.05), Value::Number(5.0), Value::Number(-100.0), Value::Number(0.0), Value::Number(0.0)];
    let args1 = [Value::Number(0.05), Value::Number(5.0), Value::Number(-100.0), Value::Number(0.0), Value::Number(1.0)];
    let pv0 = pv_fn(&args0);
    let pv1 = pv_fn(&args1);
    assert!(matches!(pv0, Value::Number(_)));
    assert!(matches!(pv1, Value::Number(_)));
    assert_ne!(pv0, pv1);
}

#[test]
fn zero_pmt_with_fv() {
    // PV(0.1, 5, 0, 100) = -100/(1.1^5) ≈ -62.09
    let args = [
        Value::Number(0.1),
        Value::Number(5.0),
        Value::Number(0.0),
        Value::Number(100.0),
    ];
    assert!(approx(pv_fn(&args), -62.09, 0.01));
}

#[test]
fn fv_defaults_to_zero() {
    let args3 = [Value::Number(0.05), Value::Number(10.0), Value::Number(-100.0)];
    let args4 = [Value::Number(0.05), Value::Number(10.0), Value::Number(-100.0), Value::Number(0.0)];
    assert_eq!(pv_fn(&args3), pv_fn(&args4));
}
