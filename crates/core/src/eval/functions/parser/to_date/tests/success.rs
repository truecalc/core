use super::super::*;
use crate::types::Value;

#[test]
fn one_converts_to_date() {
    let args = [Value::Number(1.0)];
    assert_eq!(to_date_fn(&args), Value::Date(1.0));
}

#[test]
fn large_serial_converts_to_date() {
    let args = [Value::Number(45292.0)];
    assert_eq!(to_date_fn(&args), Value::Date(45292.0));
}

#[test]
fn date_input_passthrough() {
    let args = [Value::Date(45292.0)];
    assert_eq!(to_date_fn(&args), Value::Date(45292.0));
}

#[test]
fn text_passthrough() {
    let args = [Value::Text("text".to_string())];
    assert_eq!(to_date_fn(&args), Value::Text("text".to_string()));
}
