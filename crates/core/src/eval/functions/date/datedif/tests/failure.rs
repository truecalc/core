use super::super::*;
use crate::types::{ErrorKind, Value};

fn num(serial: f64) -> Value {
    Value::Number(serial)
}

#[test]
fn too_few_args() {
    assert_eq!(datedif_fn(&[num(45292.0), num(45458.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    let args = [num(45292.0), num(45458.0), Value::Text("D".into()), num(0.0)];
    assert_eq!(datedif_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn start_greater_than_end_returns_num() {
    // DATEDIF(DATE(2024,6,15), DATE(2024,1,1), "D") → #NUM!
    let args = [num(45458.0), num(45292.0), Value::Text("D".into())];
    assert_eq!(datedif_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn invalid_unit_returns_num() {
    // DATEDIF(DATE(2024,1,1), DATE(2024,6,15), "X") → #NUM!
    let args = [num(45292.0), num(45458.0), Value::Text("X".into())];
    assert_eq!(datedif_fn(&args), Value::Error(ErrorKind::Num));
}
