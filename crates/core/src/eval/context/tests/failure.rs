use super::super::*;
use crate::types::Value;

#[test]
fn get_missing_returns_empty() {
    let ctx = Context::empty();
    assert_eq!(ctx.get("x"), Value::Empty);
}

#[test]
fn get_missing_on_non_empty_context_returns_empty() {
    let vars = [("a".to_string(), Value::Number(1.0))].into();
    let ctx = Context::new(vars);
    assert_eq!(ctx.get("b"), Value::Empty);
}
