use super::super::*;
use crate::types::Value;

#[test]
fn format_comma_grouped_with_decimals() {
    // TEXT(1234.5, "#,##0.00") → "1,234.50"
    assert_eq!(
        text_fn(&[Value::Number(1234.5), Value::Text("#,##0.00".to_string())]),
        Value::Text("1,234.50".to_string())
    );
}

#[test]
fn format_percentage() {
    // TEXT(0.25, "0%") → "25%"
    assert_eq!(
        text_fn(&[Value::Number(0.25), Value::Text("0%".to_string())]),
        Value::Text("25%".to_string())
    );
}

#[test]
fn format_zero_integer() {
    assert_eq!(
        text_fn(&[Value::Number(42.0), Value::Text("0".to_string())]),
        Value::Text("42".to_string())
    );
}

#[test]
fn format_two_decimal_places() {
    // 3.1 must be padded to "3.10", not left as "3.1"
    assert_eq!(
        text_fn(&[Value::Number(3.1), Value::Text("0.00".to_string())]),
        Value::Text("3.10".to_string())
    );
}

#[test]
fn format_two_decimal_places_rounds() {
    assert_eq!(
        text_fn(&[Value::Number(3.14159), Value::Text("0.00".to_string())]),
        Value::Text("3.14".to_string())
    );
}

#[test]
fn format_integer_rounds() {
    assert_eq!(
        text_fn(&[Value::Number(1234.6), Value::Text("0".to_string())]),
        Value::Text("1235".to_string())
    );
}

#[test]
fn format_three_decimal_places_zero() {
    assert_eq!(
        text_fn(&[Value::Number(3.0), Value::Text("0.000".to_string())]),
        Value::Text("3.000".to_string())
    );
}

#[test]
fn format_integer_truncated() {
    assert_eq!(
        text_fn(&[Value::Number(3.9), Value::Text("0".to_string())]),
        Value::Text("4".to_string())
    );
}
