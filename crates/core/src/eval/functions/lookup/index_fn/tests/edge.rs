use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn zero_index_returns_value_error() {
    // INDEX is 1-based; 0 is invalid → #VALUE!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(index_fn(&[arr, Value::Number(0.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn float_index_truncated() {
    // INDEX({10, 20, 30}, 2.9) — f64→usize truncates to 2 → 20
    let arr = Value::Array(vec![
        Value::Number(10.0),
        Value::Number(20.0),
        Value::Number(30.0),
    ]);
    assert_eq!(index_fn(&[arr, Value::Number(2.9)]), Value::Number(20.0));
}

#[test]
fn index_with_optional_col_arg() {
    // INDEX accepts an optional 3rd arg; behaviour same as 2-arg for 1-D arrays
    let arr = Value::Array(vec![
        Value::Number(10.0),
        Value::Number(20.0),
    ]);
    assert_eq!(
        index_fn(&[arr, Value::Number(1.0), Value::Number(1.0)]),
        Value::Number(10.0)
    );
}
