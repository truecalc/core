use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    crate::evaluate(formula, &HashMap::new())
}

#[test]
fn zero_args_returns_na() {
    assert_eq!(run("=SUMIFS()"), Value::Error(ErrorKind::NA));
}

#[test]
fn one_arg_returns_na() {
    assert_eq!(run("=SUMIFS(1)"), Value::Error(ErrorKind::NA));
}

#[test]
fn two_args_returns_na() {
    assert_eq!(run("=SUMIFS(1,1)"), Value::Error(ErrorKind::NA));
}

#[test]
fn even_args_after_sum_range_returns_na() {
    assert_eq!(run("=SUMIFS(1,1,1,1)"), Value::Error(ErrorKind::NA));
}
