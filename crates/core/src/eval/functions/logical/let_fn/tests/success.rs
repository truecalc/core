use crate::evaluate;
use crate::types::Value;
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

#[test]
fn let_single_binding() {
    assert_eq!(run("=LET(x, 5, x*2)"), Value::Number(10.0));
}

#[test]
fn let_two_bindings() {
    assert_eq!(run("=LET(x, 3, y, 4, x+y)"), Value::Number(7.0));
}

#[test]
fn let_later_binding_references_earlier() {
    // y = x+1 = 3, x*y = 2*3 = 6
    assert_eq!(run("=LET(x, 2, y, x+1, x*y)"), Value::Number(6.0));
}
