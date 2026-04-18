use crate::evaluate;
use crate::types::Value;
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

#[test]
fn lambda_body_uses_conditional() {
    // LAMBDA with IF inside the body
    assert_eq!(run("=LAMBDA(x, IF(x>0, x, -x))(-5)"), Value::Number(5.0));
}
