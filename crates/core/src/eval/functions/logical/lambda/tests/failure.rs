use crate::evaluate;
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

#[test]
fn lambda_bare_returns_na() {
    // LAMBDA without invocation returns #N/A
    assert_eq!(run("=LAMBDA(x, x*2)"), Value::Error(ErrorKind::NA));
}
