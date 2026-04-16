use super::averagea_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn simple_numbers() {
    assert_eq!(
        averagea_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(2.0)
    );
}

#[test]
fn bool_coercion() {
    // TRUE=1, FALSE=0 → (1+0+1)/3 = 2/3
    let result = averagea_fn(&[Value::Bool(true), Value::Bool(false), Value::Number(1.0)]);
    assert_eq!(result, Value::Number(2.0 / 3.0));
}

#[test]
fn text_direct_arg_returns_value_error() {
    assert_eq!(
        averagea_fn(&[Value::Text("text".to_string()), Value::Number(1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn empty_skipped() {
    assert_eq!(
        averagea_fn(&[Value::Empty, Value::Number(4.0), Value::Empty]),
        Value::Number(4.0)
    );
}

#[test]
fn no_args_returns_na() {
    assert_eq!(averagea_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn all_empty_returns_div_zero() {
    assert_eq!(averagea_fn(&[Value::Empty]), Value::Error(ErrorKind::DivByZero));
}
