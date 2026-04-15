use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args() {
    assert_eq!(rate_fn(&[Value::Number(5.0), Value::Number(-100.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn too_many_args() {
    let args = vec![Value::Number(0.1), Value::Number(0.1), Value::Number(0.1),
                    Value::Number(0.1), Value::Number(0.1), Value::Number(0.1),
                    Value::Number(0.1)];
    assert_eq!(rate_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_nper() {
    let args = [Value::Text("bad".to_string()), Value::Number(-100.0), Value::Number(1000.0)];
    assert_eq!(rate_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn no_convergence_bad_inputs() {
    // Nonsensical inputs: pv=0, pmt=0, fv=0 — no convergence
    let args = [Value::Number(5.0), Value::Number(0.0), Value::Number(0.0)];
    // May converge to 0 or fail — just confirm it returns Number or Num error
    let result = rate_fn(&args);
    assert!(matches!(result, Value::Number(_) | Value::Error(ErrorKind::Num)));
}
