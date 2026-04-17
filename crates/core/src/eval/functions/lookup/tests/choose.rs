use crate::evaluate;
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

// CHOOSE(1, "a", "b", "c") → "a"
#[test]
fn choose_first() {
    assert_eq!(run(r#"CHOOSE(1, "a", "b", "c")"#), Value::Text("a".to_string()));
}

// CHOOSE(3, "x", "y", "z") → "z"
#[test]
fn choose_last() {
    assert_eq!(run(r#"CHOOSE(3, "x", "y", "z")"#), Value::Text("z".to_string()));
}

// CHOOSE(2, 10, 20, 30) → 20
#[test]
fn choose_middle_number() {
    assert_eq!(run("CHOOSE(2, 10, 20, 30)"), Value::Number(20.0));
}

// CHOOSE(0, "a", "b") → #NUM! (out of range: index < 1)
#[test]
fn choose_index_zero_is_error() {
    assert_eq!(run(r#"CHOOSE(0, "a", "b")"#), Value::Error(ErrorKind::Num));
}

// CHOOSE(5, "a", "b") → #NUM! (index > number of choices)
#[test]
fn choose_index_too_large_is_error() {
    assert_eq!(run(r#"CHOOSE(5, "a", "b")"#), Value::Error(ErrorKind::Num));
}
