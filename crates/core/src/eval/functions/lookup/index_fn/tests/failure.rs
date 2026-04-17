use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(index_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn one_arg_returns_na() {
    let arr = Value::Array(vec![Value::Number(1.0)]);
    assert_eq!(index_fn(&[arr]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let arr = Value::Array(vec![Value::Number(1.0)]);
    assert_eq!(
        index_fn(&[arr, Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn out_of_bounds_returns_ref_error() {
    // INDEX({10, 20}, 5) → #REF!
    let arr = Value::Array(vec![Value::Number(10.0), Value::Number(20.0)]);
    assert_eq!(index_fn(&[arr, Value::Number(5.0)]), Value::Error(ErrorKind::Ref));
}

#[test]
fn scalar_at_index_gt_one_returns_ref_error() {
    // INDEX(42, 2) → #REF! (scalar only valid at index 1)
    assert_eq!(
        index_fn(&[Value::Number(42.0), Value::Number(2.0)]),
        Value::Error(ErrorKind::Ref)
    );
}
