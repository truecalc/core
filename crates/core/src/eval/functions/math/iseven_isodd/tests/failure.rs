use super::super::*;
use crate::types::{ErrorKind, Value};

// ── ISEVEN ────────────────────────────────────────────────────────────────────

#[test]
fn iseven_text_returns_value_error() {
    assert_eq!(
        iseven_fn(&[Value::Text("text".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn iseven_no_args_returns_na_error() {
    assert_eq!(iseven_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn iseven_too_many_args_returns_na_error() {
    assert_eq!(
        iseven_fn(&[Value::Number(2.0), Value::Number(3.0)]),
        Value::Error(ErrorKind::NA)
    );
}

// ── ISODD ─────────────────────────────────────────────────────────────────────

#[test]
fn isodd_text_returns_value_error() {
    assert_eq!(
        isodd_fn(&[Value::Text("text".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn isodd_no_args_returns_na_error() {
    assert_eq!(isodd_fn(&[]), Value::Error(ErrorKind::NA));
}
