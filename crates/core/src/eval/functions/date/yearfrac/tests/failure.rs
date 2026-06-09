use super::super::yearfrac_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(yearfrac_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn one_arg_returns_na() {
    assert_eq!(yearfrac_fn(&[Value::Number(45292.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let args = [
        Value::Number(45292.0), Value::Number(45658.0),
        Value::Number(0.0), Value::Number(0.0),
    ];
    assert_eq!(yearfrac_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn non_numeric_start_returns_value_error() {
    let args = [Value::Text("bad".to_string()), Value::Number(45658.0)];
    assert_eq!(yearfrac_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn invalid_basis_5_returns_num_error() {
    let args = [Value::Number(45292.0), Value::Number(45658.0), Value::Number(5.0)];
    assert_eq!(yearfrac_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn negative_basis_returns_num_error() {
    // =YEARFRAC(DATE(2023,1,1),DATE(2023,12,31),-1) → #NUM!
    let args = [Value::Number(44927.0), Value::Number(45291.0), Value::Number(-1.0)];
    assert_eq!(yearfrac_fn(&args), Value::Error(ErrorKind::Num));
}
