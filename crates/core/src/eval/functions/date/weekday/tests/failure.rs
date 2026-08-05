use super::super::weekday_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(weekday_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let args = [Value::Number(45292.0), Value::Number(1.0), Value::Number(1.0)];
    assert_eq!(weekday_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn non_numeric_date_returns_value_error() {
    let args = [Value::Text("bad".to_string())];
    assert_eq!(weekday_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn invalid_return_type_8_returns_num_error() {
    // =WEEKDAY(DATE(2024,1,1),8) → #NUM!
    let args = [Value::Number(45292.0), Value::Number(8.0)];
    assert_eq!(weekday_fn(&args), Value::Error(ErrorKind::Num));
}
