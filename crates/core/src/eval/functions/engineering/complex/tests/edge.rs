use super::super::{complex_fn, imaginary_fn, imreal_fn};
use crate::types::Value;

fn text(s: &str) -> Value { Value::Text(s.to_string()) }

#[test]
fn complex_negative_both() {
    // COMPLEX(-3, -4) → "-3-4i"
    assert_eq!(
        complex_fn(&[Value::Number(-3.0), Value::Number(-4.0)]),
        text("-3-4i")
    );
}

#[test]
fn complex_unit_imaginary_only() {
    // COMPLEX(0, 1) → "i"
    assert_eq!(
        complex_fn(&[Value::Number(0.0), Value::Number(1.0)]),
        text("i")
    );
}

#[test]
fn complex_negative_unit_imaginary() {
    // COMPLEX(0, -1) → "-i"
    assert_eq!(
        complex_fn(&[Value::Number(0.0), Value::Number(-1.0)]),
        text("-i")
    );
}

#[test]
fn imreal_pure_imaginary_returns_zero() {
    // "3i" has no real part
    assert_eq!(imreal_fn(&[text("3i")]), Value::Number(0.0));
}

#[test]
fn imaginary_pure_real_returns_zero() {
    // "5" has no imaginary part
    assert_eq!(imaginary_fn(&[text("5")]), Value::Number(0.0));
}
