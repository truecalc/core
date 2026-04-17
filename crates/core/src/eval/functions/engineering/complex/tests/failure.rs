use super::super::{complex_fn, imabs_fn};
use crate::types::{ErrorKind, Value};

fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}

#[test]
fn complex_non_numeric_real() {
    // Non-numeric first arg → #VALUE!
    assert_eq!(
        complex_fn(&[text("abc"), Value::Number(1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn complex_non_numeric_imag() {
    // Non-numeric second arg → #VALUE!
    assert_eq!(
        complex_fn(&[Value::Number(1.0), text("abc")]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn complex_invalid_suffix() {
    // Suffix other than "i" or "j" → #VALUE!
    assert_eq!(
        complex_fn(&[Value::Number(1.0), Value::Number(2.0), text("k")]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn imabs_invalid_string() {
    // Unparseable complex string → #VALUE!
    assert_eq!(
        imabs_fn(&[text("not-a-number")]),
        Value::Error(ErrorKind::Value)
    );
}
