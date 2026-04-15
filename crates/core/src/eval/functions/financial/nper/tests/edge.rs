use super::super::*;
use crate::types::{ErrorKind, Value};

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn type1_beginning_of_period() {
    let args = [
        Value::Number(0.1),
        Value::Number(-100.0),
        Value::Number(500.0),
        Value::Number(0.0),
        Value::Number(1.0),
    ];
    let result = nper_fn(&args);
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn impossible_ratio_returns_num() {
    // pmt=0, pv=1000, fv=10000, rate=0.1 → ratio = -10 → #NUM!
    let args = [
        Value::Number(0.1),
        Value::Number(0.0),
        Value::Number(1000.0),
        Value::Number(10000.0),
    ];
    assert_eq!(nper_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn fv_defaults_to_zero() {
    let args3 = [Value::Number(0.05), Value::Number(-100.0), Value::Number(500.0)];
    let args4 = [Value::Number(0.05), Value::Number(-100.0), Value::Number(500.0), Value::Number(0.0)];
    assert!(approx(nper_fn(&args3), nper_fn(&args4).into_f64(), 1e-9));
}

trait IntoF64 {
    fn into_f64(self) -> f64;
}

impl IntoF64 for Value {
    fn into_f64(self) -> f64 {
        if let Value::Number(n) = self { n } else { f64::NAN }
    }
}
