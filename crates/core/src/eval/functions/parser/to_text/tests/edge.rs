use super::super::*;
use crate::types::Value;

#[test]
fn zero_to_text() {
    let args = [Value::Number(0.0)];
    assert_eq!(to_text_fn(&args), Value::Text("0".to_string()));
}

#[test]
fn negative_to_text() {
    let args = [Value::Number(-5.0)];
    assert_eq!(to_text_fn(&args), Value::Text("-5".to_string()));
}

#[test]
fn date_serial_to_text() {
    let args = [Value::Date(45292.0)];
    assert_eq!(to_text_fn(&args), Value::Text("45292".to_string()));
}
