use crate::evaluate;
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

#[test]
fn let_even_arg_count_returns_na() {
    // Even number of args means no body — should return #N/A
    assert_eq!(run("=LET(x, 5)"), Value::Error(ErrorKind::NA));
}

#[test]
fn let_too_few_args_returns_na() {
    // Single arg is not enough for even one binding
    assert_eq!(run("=LET(x)"), Value::Error(ErrorKind::NA));
}
