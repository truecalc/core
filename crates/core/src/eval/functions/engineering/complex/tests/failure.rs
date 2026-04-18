use super::super::{complex_fn, imabs_fn, imaginary_fn, imreal_fn};
use crate::types::{ErrorKind, Value};

fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}

// ── Non-parseable string → Error(Value) ───────────────────────────────────────

#[test]
fn imabs_non_parseable() {
    assert_eq!(imabs_fn(&[t("not-a-number")]), Value::Error(ErrorKind::Value));
}

#[test]
fn imreal_non_parseable() {
    assert_eq!(
        imreal_fn(&[t("bad input")]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn imaginary_non_parseable() {
    assert_eq!(
        imaginary_fn(&[t("??")]),
        Value::Error(ErrorKind::Value)
    );
}

// ── Non-numeric args to COMPLEX → Error(Value) ────────────────────────────────

#[test]
fn complex_non_numeric_real() {
    // Non-numeric first arg → #VALUE!
    assert_eq!(
        complex_fn(&[t("abc"), Value::Number(1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn complex_non_numeric_imag() {
    // Non-numeric second arg → #VALUE!
    assert_eq!(
        complex_fn(&[Value::Number(1.0), t("abc")]),
        Value::Error(ErrorKind::Value)
    );
}

// ── Arity errors ──────────────────────────────────────────────────────────────

#[test]
fn complex_no_args() {
    assert_eq!(complex_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn complex_one_arg() {
    assert_eq!(
        complex_fn(&[Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn imabs_no_args() {
    assert_eq!(imabs_fn(&[]), Value::Error(ErrorKind::NA));
}

// ── Invalid suffix → Error(Value) ────────────────────────────────────────────

#[test]
fn complex_invalid_suffix() {
    assert_eq!(
        complex_fn(&[Value::Number(1.0), Value::Number(2.0), t("k")]),
        Value::Error(ErrorKind::Value)
    );
}
