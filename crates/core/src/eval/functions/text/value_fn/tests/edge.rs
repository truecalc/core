use super::super::*;
use crate::types::Value;

#[test]
fn whitespace_trimmed() {
    assert_eq!(
        value_fn(&[Value::Text("  42  ".to_string())]),
        Value::Number(42.0)
    );
}

#[test]
fn empty_string_returns_zero() {
    // Google Sheets: VALUE("") → 0
    assert_eq!(
        value_fn(&[Value::Text("".to_string())]),
        Value::Number(0.0)
    );
}

#[test]
fn number_passthrough() {
    // Number coerced to string "42" then re-parsed
    assert_eq!(
        value_fn(&[Value::Number(42.0)]),
        Value::Number(42.0)
    );
}
