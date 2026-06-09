use super::super::networkdays_intl_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args() {
    assert_eq!(networkdays_intl_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn one_arg() {
    assert_eq!(networkdays_intl_fn(&[Value::Number(45292.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    let args = [
        Value::Number(45292.0),
        Value::Number(45296.0),
        Value::Number(1.0),
        Value::Number(0.0),
        Value::Number(0.0),
    ];
    assert_eq!(networkdays_intl_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn non_numeric_start() {
    let args = [Value::Text("bad".to_string()), Value::Number(45296.0)];
    assert_eq!(networkdays_intl_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn invalid_weekend_code() {
    let args = [Value::Number(45292.0), Value::Number(45296.0), Value::Number(99.0)];
    assert_eq!(networkdays_intl_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn all_ones_string_weekend_returns_num_error() {
    // =NETWORKDAYS.INTL(...,"1111111") → #NUM!
    let args = [Value::Number(45292.0), Value::Number(45298.0), Value::Text("1111111".into())];
    assert_eq!(networkdays_intl_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn invalid_chars_in_string_weekend_returns_num_error() {
    // =NETWORKDAYS.INTL(...,"abc0011") → #NUM!
    let args = [Value::Number(45292.0), Value::Number(45298.0), Value::Text("abc0011".into())];
    assert_eq!(networkdays_intl_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn wrong_length_string_weekend_returns_num_error() {
    // =NETWORKDAYS.INTL(...,"00011") → #NUM!
    let args = [Value::Number(45292.0), Value::Number(45298.0), Value::Text("00011".into())];
    assert_eq!(networkdays_intl_fn(&args), Value::Error(ErrorKind::Num));
}
