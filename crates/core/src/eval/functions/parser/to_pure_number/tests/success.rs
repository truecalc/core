use super::super::*;
use crate::types::Value;

#[test]
fn integer_passthrough() {
    let args = [Value::Number(42.0)];
    assert_eq!(to_pure_number_fn(&args), Value::Number(42.0));
}

#[test]
fn decimal_passthrough() {
    let args = [Value::Number(3.14)];
    assert_eq!(to_pure_number_fn(&args), Value::Number(3.14));
}

#[test]
fn date_strips_to_number() {
    let args = [Value::Date(45292.0)];
    assert_eq!(to_pure_number_fn(&args), Value::Number(45292.0));
}

#[test]
fn text_passthrough() {
    let args = [Value::Text("text".to_string())];
    assert_eq!(to_pure_number_fn(&args), Value::Text("text".to_string()));
}
