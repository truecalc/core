use super::super::concat_fn;
use crate::types::{ErrorKind, Value};

// ── basic concatenation ───────────────────────────────────────────────────────

#[test]
fn two_text_args_returns_text() {
    // CONCAT("hello", " world") → "hello world"
    assert_eq!(
        concat_fn(&[Value::Text("hello".into()), Value::Text(" world".into())]),
        Value::Text("hello world".into())
    );
}

#[test]
fn numeric_args_returns_text_not_number() {
    // CONCAT(1, 2) must return Text("12"), never Number(12.0)
    assert_eq!(
        concat_fn(&[Value::Number(1.0), Value::Number(2.0)]),
        Value::Text("12".into())
    );
}

#[test]
fn mixed_number_and_text_returns_text() {
    // CONCAT("x", 9) → "x9"
    assert_eq!(
        concat_fn(&[Value::Text("x".into()), Value::Number(9.0)]),
        Value::Text("x9".into())
    );
}

// ── arity errors ──────────────────────────────────────────────────────────────

#[test]
fn too_few_args_returns_na_error() {
    assert_eq!(
        concat_fn(&[Value::Text("a".into())]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn too_many_args_returns_na_error() {
    assert_eq!(
        concat_fn(&[
            Value::Text("a".into()),
            Value::Text("b".into()),
            Value::Text("c".into()),
        ]),
        Value::Error(ErrorKind::NA)
    );
}
