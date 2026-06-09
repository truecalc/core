use super::super::*;
use crate::types::Value;

#[test]
fn negative_number_integer_format() {
    assert_eq!(
        text_fn(&[Value::Number(-5.7), Value::Text("0".to_string())]),
        Value::Text("-6".to_string())
    );
}

#[test]
fn zero_integer_format() {
    assert_eq!(
        text_fn(&[Value::Number(0.0), Value::Text("0".to_string())]),
        Value::Text("0".to_string())
    );
}

#[test]
fn bool_not_coerced_returns_text() {
    // GS: TEXT(TRUE, ...) returns "TRUE" — boolean is NOT coerced to a number
    assert_eq!(
        text_fn(&[Value::Bool(true), Value::Text("0".to_string())]),
        Value::Text("TRUE".to_string())
    );
}

#[test]
fn unsupported_format_falls_back_to_display_number() {
    // "General" is not a recognised pattern; should fall back to display_number
    assert_eq!(
        text_fn(&[Value::Number(3.5), Value::Text("General".to_string())]),
        Value::Text("3.5".to_string())
    );
}

#[test]
fn hash_format_suppresses_trailing_zeros() {
    // "0.##" — # suppresses trailing zeros; 1.5 -> "1.5" (not "1.50")
    assert_eq!(
        text_fn(&[Value::Number(1.5), Value::Text("0.##".to_string())]),
        Value::Text("1.5".to_string())
    );
}
