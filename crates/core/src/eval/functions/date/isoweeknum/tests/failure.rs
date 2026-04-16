use super::super::isoweeknum_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    // =ISOWEEKNUM() → #N/A
    assert_eq!(isoweeknum_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let args = [Value::Number(45292.0), Value::Number(1.0)];
    assert_eq!(isoweeknum_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn non_numeric_arg_returns_value_error() {
    let args = [Value::Text("not_a_date".to_string())];
    assert_eq!(isoweeknum_fn(&args), Value::Error(ErrorKind::Value));
}
