use super::super::averageif_fn;
use crate::types::{ErrorKind, Value};

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(averageif_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_one_arg() {
    assert_eq!(
        averageif_fn(&[Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn wrong_arity_four_args() {
    assert_eq!(
        averageif_fn(&[
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
        ]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn no_matches_returns_div_by_zero() {
    // AVERAGEIF({1,2,3}, ">10") → #DIV/0!
    let result = averageif_fn(&[nums(&[1.0, 2.0, 3.0]), Value::Text(">10".to_string())]);
    assert_eq!(result, Value::Error(ErrorKind::DivByZero));
}

#[test]
fn empty_array_returns_div_by_zero() {
    let result = averageif_fn(&[Value::Array(vec![]), Value::Number(1.0)]);
    assert_eq!(result, Value::Error(ErrorKind::DivByZero));
}
