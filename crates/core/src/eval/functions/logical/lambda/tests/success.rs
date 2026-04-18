use crate::evaluate;
use crate::types::Value;
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

#[test]
fn lambda_single_arg() {
    assert_eq!(run("=LAMBDA(x, x*2)(5)"), Value::Number(10.0));
}

#[test]
fn lambda_two_args() {
    assert_eq!(run("=LAMBDA(x, y, x+y)(3, 4)"), Value::Number(7.0));
}

#[test]
fn lambda_square() {
    assert_eq!(run("=LAMBDA(x, x*x)(3)"), Value::Number(9.0));
}
