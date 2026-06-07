use crate::Engine;
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    Engine::sheets().evaluate(formula, &HashMap::new())
}

// ── basic concatenation ───────────────────────────────────────────────────────

#[test]
fn two_text_args_returns_text() {
    assert_eq!(run(r#"=CONCAT("hello", " world")"#), Value::Text("hello world".into()));
}

#[test]
fn numeric_args_returns_text_not_number() {
    assert_eq!(run("=CONCAT(1, 2)"), Value::Text("12".into()));
}

#[test]
fn mixed_number_and_text_returns_text() {
    assert_eq!(run(r#"=CONCAT("x", 9)"#), Value::Text("x9".into()));
}

// ── arity errors ──────────────────────────────────────────────────────────────

#[test]
fn too_few_args_returns_na_error() {
    assert_eq!(run(r#"=CONCAT("a")"#), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na_error() {
    assert_eq!(run(r#"=CONCAT("a", "b", "c")"#), Value::Error(ErrorKind::NA));
}
