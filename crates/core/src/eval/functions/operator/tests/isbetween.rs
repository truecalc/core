use super::super::isbetween_fn;
use crate::types::{ErrorKind, Value};

// ── inclusive bounds (defaults) ───────────────────────────────────────────────

#[test]
fn value_within_range_returns_true() {
    // ISBETWEEN(3, 1, 5) → true
    assert_eq!(
        isbetween_fn(&[Value::Number(3.0), Value::Number(1.0), Value::Number(5.0)]),
        Value::Bool(true)
    );
}

#[test]
fn value_at_lower_bound_inclusive_returns_true() {
    // ISBETWEEN(1, 1, 5) → true (inclusive lower bound by default)
    assert_eq!(
        isbetween_fn(&[Value::Number(1.0), Value::Number(1.0), Value::Number(5.0)]),
        Value::Bool(true)
    );
}

#[test]
fn value_at_upper_bound_inclusive_returns_true() {
    // ISBETWEEN(5, 1, 5) → true (inclusive upper bound by default)
    assert_eq!(
        isbetween_fn(&[Value::Number(5.0), Value::Number(1.0), Value::Number(5.0)]),
        Value::Bool(true)
    );
}

#[test]
fn value_below_lower_bound_returns_false() {
    // ISBETWEEN(0, 1, 5) → false
    assert_eq!(
        isbetween_fn(&[Value::Number(0.0), Value::Number(1.0), Value::Number(5.0)]),
        Value::Bool(false)
    );
}

#[test]
fn value_above_upper_bound_returns_false() {
    // ISBETWEEN(6, 1, 5) → false
    assert_eq!(
        isbetween_fn(&[Value::Number(6.0), Value::Number(1.0), Value::Number(5.0)]),
        Value::Bool(false)
    );
}

// ── exclusive lower bound ─────────────────────────────────────────────────────

#[test]
fn value_at_lower_bound_exclusive_returns_false() {
    // ISBETWEEN(1, 1, 5, FALSE) → false (exclusive lower bound)
    assert_eq!(
        isbetween_fn(&[
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(5.0),
            Value::Bool(false),
        ]),
        Value::Bool(false)
    );
}

// ── exclusive upper bound ─────────────────────────────────────────────────────

#[test]
fn value_at_upper_bound_exclusive_returns_false() {
    // ISBETWEEN(5, 1, 5, TRUE, FALSE) → false (exclusive upper bound)
    assert_eq!(
        isbetween_fn(&[
            Value::Number(5.0),
            Value::Number(1.0),
            Value::Number(5.0),
            Value::Bool(true),
            Value::Bool(false),
        ]),
        Value::Bool(false)
    );
}

// ── arity errors ──────────────────────────────────────────────────────────────

#[test]
fn too_few_args_returns_na_error() {
    assert_eq!(
        isbetween_fn(&[Value::Number(3.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn too_many_args_returns_na_error() {
    assert_eq!(
        isbetween_fn(&[
            Value::Number(3.0),
            Value::Number(1.0),
            Value::Number(5.0),
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(true),
        ]),
        Value::Error(ErrorKind::NA)
    );
}
