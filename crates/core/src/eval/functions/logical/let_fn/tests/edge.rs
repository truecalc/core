use crate::evaluate;
use crate::types::Value;
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

#[test]
fn let_body_is_literal() {
    // Minimum valid: one binding, body is just a number
    assert_eq!(run("=LET(x, 42, 99)"), Value::Number(99.0));
}

#[test]
fn let_binding_shadows_outer() {
    // x is only in scope inside the body
    let mut vars = HashMap::new();
    vars.insert("X".to_string(), crate::types::Value::Number(1.0));
    // LET(x, 10, x) should return 10, not the outer x=1
    assert_eq!(evaluate("=LET(x, 10, x)", &vars), Value::Number(10.0));
}
