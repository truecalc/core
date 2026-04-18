use super::super::*;
use crate::types::Value;

#[test]
fn get_existing_variable() {
    let vars = [("x".to_string(), Value::Number(42.0))].into();
    let ctx = Context::new(vars);
    assert_eq!(ctx.get("x"), Value::Number(42.0));
}

#[test]
fn get_case_insensitive() {
    let vars = [("myVar".to_string(), Value::Number(42.0))].into();
    let ctx = Context::new(vars);
    // All casing variants should find the same value
    assert_eq!(ctx.get("myVar"), Value::Number(42.0));
    assert_eq!(ctx.get("MYVAR"), Value::Number(42.0));
    assert_eq!(ctx.get("myvar"), Value::Number(42.0));
}

#[test]
fn new_normalises_keys_to_uppercase() {
    let vars = [("lower".to_string(), Value::Bool(true))].into();
    let ctx = Context::new(vars);
    assert_eq!(ctx.get("LOWER"), Value::Bool(true));
}
