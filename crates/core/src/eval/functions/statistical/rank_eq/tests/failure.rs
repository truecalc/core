use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn rank_eq_x_not_in_ref_returns_na() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(rank_eq_fn(&[Value::Number(5.0), arr]), Value::Error(ErrorKind::NA));
}

#[test]
fn rank_eq_empty_ref_returns_na() {
    let arr = Value::Array(vec![]);
    assert_eq!(rank_eq_fn(&[Value::Number(1.0), arr]), Value::Error(ErrorKind::NA));
}

#[test]
fn rank_eq_non_number_x_returns_na() {
    let arr = Value::Array(vec![Value::Number(1.0)]);
    assert_eq!(
        rank_eq_fn(&[Value::Text("a".to_string()), arr]),
        Value::Error(ErrorKind::NA)
    );
}
