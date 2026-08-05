use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(bitrshift_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn one_arg_returns_na() {
    assert_eq!(bitrshift_fn(&[Value::Number(1.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn negative_number_returns_num() {
    assert_eq!(bitrshift_fn(&[Value::Number(-1.0), Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn exceeds_max_input_returns_num() {
    let too_big = 281_474_976_710_656.0_f64;
    assert_eq!(bitrshift_fn(&[Value::Number(too_big), Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn result_exceeds_max_returns_num() {
    let max = 281_474_976_710_655.0_f64;
    assert_eq!(bitrshift_fn(&[Value::Number(max), Value::Number(-1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn non_numeric_returns_value() {
    assert_eq!(bitrshift_fn(&[Value::Text("abc".to_string()), Value::Number(1.0)]), Value::Error(ErrorKind::Value));
}
