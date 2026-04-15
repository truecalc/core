use super::super::*;
use crate::types::Value;

#[test]
fn empty_context_has_no_bindings() {
    let ctx = Context::empty();
    assert!(ctx.vars.is_empty());
}

#[test]
fn new_with_empty_map_is_equivalent_to_empty() {
    let ctx = Context::new(std::collections::HashMap::new());
    assert!(ctx.vars.is_empty());
}

#[test]
fn get_on_mixed_case_key_stored_as_uppercase() {
    // Keys stored as uppercase; the stored key in vars is "MYKEY"
    let vars = [("MyKey".to_string(), Value::Text("v".to_string()))].into();
    let ctx = Context::new(vars);
    assert!(ctx.vars.contains_key("MYKEY"));
    assert!(!ctx.vars.contains_key("MyKey"));
}
