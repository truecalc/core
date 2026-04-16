use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn rank_x_not_in_ref_returns_na() {
    // x=6 not in [1,2,3,4,5] → #N/A
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_fn(&[Value::Number(6.0), arr]), Value::Error(ErrorKind::NA));
}

#[test]
fn rank_empty_ref_returns_na() {
    // Empty array → #N/A
    let arr = Value::Array(vec![]);
    assert_eq!(rank_fn(&[Value::Number(1.0), arr]), Value::Error(ErrorKind::NA));
}

#[test]
fn rank_non_number_x_returns_na() {
    // x is text → #N/A
    let arr = Value::Array(vec![Value::Number(1.0)]);
    assert_eq!(
        rank_fn(&[Value::Text("a".to_string()), arr]),
        Value::Error(ErrorKind::NA)
    );
}
