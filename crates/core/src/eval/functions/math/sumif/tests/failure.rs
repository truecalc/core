use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    crate::evaluate(formula, &HashMap::new())
}

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(run("=SUMIF()"), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_one_arg() {
    assert_eq!(run("=SUMIF(1)"), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_four_args() {
    assert_eq!(run("=SUMIF(1,1,1,1)"), Value::Error(ErrorKind::NA));
}
