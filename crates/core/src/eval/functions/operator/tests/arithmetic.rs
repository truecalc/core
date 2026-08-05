use super::super::{add_fn, divide_fn, minus_fn, multiply_fn, pow_fn, uminus_fn};
use crate::types::{ErrorKind, Value};

fn n(v: f64) -> Value {
    Value::Number(v)
}
fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}

// ── ADD ──────────────────────────────────────────────────────────────────────

#[test]
fn add_two_numbers() {
    assert_eq!(add_fn(&[n(2.0), n(3.0)]), n(5.0));
}

#[test]
fn add_empty_string_treated_as_zero() {
    // Empty string coerces to 0 per to_number_arith
    assert_eq!(add_fn(&[t(""), n(5.0)]), n(5.0));
}

#[test]
fn add_non_numeric_text_returns_value_error() {
    assert_eq!(add_fn(&[t("abc"), n(1.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn add_wrong_arity_returns_na() {
    assert_eq!(add_fn(&[n(1.0)]), Value::Error(ErrorKind::NA));
    assert_eq!(add_fn(&[n(1.0), n(2.0), n(3.0)]), Value::Error(ErrorKind::NA));
}

// ── MINUS ────────────────────────────────────────────────────────────────────

#[test]
fn minus_two_numbers() {
    assert_eq!(minus_fn(&[n(5.0), n(3.0)]), n(2.0));
}

#[test]
fn minus_negative_result() {
    assert_eq!(minus_fn(&[n(1.0), n(4.0)]), n(-3.0));
}

#[test]
fn minus_wrong_arity_returns_na() {
    assert_eq!(minus_fn(&[n(1.0)]), Value::Error(ErrorKind::NA));
    assert_eq!(minus_fn(&[n(1.0), n(2.0), n(3.0)]), Value::Error(ErrorKind::NA));
}

// ── MULTIPLY ─────────────────────────────────────────────────────────────────

#[test]
fn multiply_two_numbers() {
    assert_eq!(multiply_fn(&[n(3.0), n(4.0)]), n(12.0));
}

#[test]
fn multiply_by_zero() {
    assert_eq!(multiply_fn(&[n(9999.0), n(0.0)]), n(0.0));
}

#[test]
fn multiply_wrong_arity_returns_na() {
    assert_eq!(multiply_fn(&[n(1.0)]), Value::Error(ErrorKind::NA));
}

// ── DIVIDE ───────────────────────────────────────────────────────────────────

#[test]
fn divide_two_numbers() {
    assert_eq!(divide_fn(&[n(10.0), n(2.0)]), n(5.0));
}

#[test]
fn divide_by_zero_returns_div_by_zero() {
    assert_eq!(divide_fn(&[n(5.0), n(0.0)]), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn divide_wrong_arity_returns_na() {
    assert_eq!(divide_fn(&[n(1.0)]), Value::Error(ErrorKind::NA));
}

// ── POW ──────────────────────────────────────────────────────────────────────

#[test]
fn pow_two_numbers() {
    assert_eq!(pow_fn(&[n(2.0), n(10.0)]), n(1024.0));
}

#[test]
fn pow_wrong_arity_returns_na() {
    assert_eq!(pow_fn(&[n(2.0)]), Value::Error(ErrorKind::NA));
}

// ── UMINUS ───────────────────────────────────────────────────────────────────

#[test]
fn uminus_negates_number() {
    assert_eq!(uminus_fn(&[n(7.0)]), n(-7.0));
}

#[test]
fn uminus_double_negation() {
    assert_eq!(uminus_fn(&[n(-3.0)]), n(3.0));
}

#[test]
fn uminus_wrong_arity_returns_na() {
    assert_eq!(uminus_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(uminus_fn(&[n(1.0), n(2.0)]), Value::Error(ErrorKind::NA));
}

// ── Overflow → #NUM! ─────────────────────────────────────────────────────────

#[test]
fn add_overflow_returns_num_error() {
    // 1e308 + 1e308 overflows f64 to infinity
    assert_eq!(add_fn(&[n(1e308), n(1e308)]), Value::Error(ErrorKind::Num));
}

#[test]
fn minus_overflow_returns_num_error() {
    // 1e308 - (-1e308) = 1e308 + 1e308 → overflow
    assert_eq!(minus_fn(&[n(1e308), n(-1e308)]), Value::Error(ErrorKind::Num));
}

#[test]
fn multiply_overflow_returns_num_error() {
    // 1e308 * 2 overflows f64
    assert_eq!(multiply_fn(&[n(1e308), n(2.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn pow_overflow_returns_num_error() {
    // 1e308 ^ 2 overflows f64
    assert_eq!(pow_fn(&[n(1e308), n(2.0)]), Value::Error(ErrorKind::Num));
}
