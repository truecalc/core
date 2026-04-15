// Test helpers for ganit-core integration tests.
use ganit_core::{evaluate, Value};
use std::collections::HashMap;

/// Convenience: evaluate a formula with no variables.
pub fn eval(formula: &str) -> Value {
    evaluate(formula, HashMap::new())
}

/// Convenience: evaluate a formula with string-keyed variables.
pub fn eval_with(formula: &str, vars: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    let mut map = HashMap::new();
    for (k, v) in vars {
        map.insert(k.to_string(), v);
    }
    evaluate(formula, map)
}
