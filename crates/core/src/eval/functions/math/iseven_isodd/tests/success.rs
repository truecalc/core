use super::super::*;
use crate::types::Value;

// ── ISEVEN ────────────────────────────────────────────────────────────────────

#[test]
fn iseven_even_positive() {
    assert_eq!(iseven_fn(&[Value::Number(2.0)]), Value::Bool(true));
}

#[test]
fn iseven_odd_positive() {
    assert_eq!(iseven_fn(&[Value::Number(3.0)]), Value::Bool(false));
}

#[test]
fn iseven_truncates_fraction() {
    // 2.9 truncates to 2, which is even
    assert_eq!(iseven_fn(&[Value::Number(2.9)]), Value::Bool(true));
}

#[test]
fn iseven_zero() {
    assert_eq!(iseven_fn(&[Value::Number(0.0)]), Value::Bool(true));
}

#[test]
fn iseven_even_negative() {
    assert_eq!(iseven_fn(&[Value::Number(-2.0)]), Value::Bool(true));
}

#[test]
fn iseven_odd_negative() {
    assert_eq!(iseven_fn(&[Value::Number(-3.0)]), Value::Bool(false));
}

// ── ISODD ─────────────────────────────────────────────────────────────────────

#[test]
fn isodd_odd_positive() {
    assert_eq!(isodd_fn(&[Value::Number(3.0)]), Value::Bool(true));
}

#[test]
fn isodd_even_positive() {
    assert_eq!(isodd_fn(&[Value::Number(2.0)]), Value::Bool(false));
}

#[test]
fn isodd_truncates_fraction() {
    // 3.9 truncates to 3, which is odd
    assert_eq!(isodd_fn(&[Value::Number(3.9)]), Value::Bool(true));
}
