use crate::Engine;
use crate::types::Value;
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    Engine::sheets().evaluate(formula, &HashMap::new())
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
    assert_eq!(Engine::sheets().evaluate("=LET(x, 10, x)", &vars), Value::Number(10.0));
}

#[test]
fn let_with_dollar_shaped_param_round_trips() {
    // $A$1 is now a syntactically legal bare identifier (issue #708), so
    // it's also legal as a LET parameter name. The binding must be set and
    // read under the same key, or this would silently miss and fall
    // through to resolving the real cell A1 instead of returning 6.
    assert_eq!(run("=LET($A$1, 5, $A$1+1)"), Value::Number(6.0));
}
