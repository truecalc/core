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
fn bool_coerced_to_number() {
    assert_eq!(
        text_fn(&[Value::Bool(true), Value::Text("0".to_string())]),
        Value::Text("1".to_string())
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
fn hash_format_pads_decimal_places() {
    // "0.##" — # treated same as 0 → two decimal places
    assert_eq!(
        text_fn(&[Value::Number(1.5), Value::Text("0.##".to_string())]),
        Value::Text("1.50".to_string())
    );
}
