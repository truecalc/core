use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn non_numeric_text() {
    assert_eq!(
        sum_fn(&[Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn first_error_wins() {
    assert_eq!(
        sum_fn(&[Value::Error(ErrorKind::Ref), Value::Error(ErrorKind::Name)]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(sum_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn wrong_arity_256_args() {
    let args: Vec<Value> = vec![Value::Number(1.0); 256];
    assert_eq!(sum_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_text_in_multi_arg_stops_at_error() {
    // SUM(1, "abc", 2) → error at arg 2 (first error wins, arg 3 not evaluated)
    assert_eq!(
        sum_fn(&[Value::Number(1.0), Value::Text("abc".to_string()), Value::Number(2.0)]),
        Value::Error(ErrorKind::Value)
    );
}
