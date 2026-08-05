use super::super::*;
use crate::types::Value;

#[test]
fn half_passthrough() {
    let args = [Value::Number(0.5)];
    assert_eq!(to_percent_fn(&args), Value::Number(0.5));
}

#[test]
fn one_passthrough() {
    let args = [Value::Number(1.0)];
    assert_eq!(to_percent_fn(&args), Value::Number(1.0));
}

#[test]
fn small_value() {
    let args = [Value::Number(0.001)];
    assert_eq!(to_percent_fn(&args), Value::Number(0.001));
}

#[test]
fn large_value() {
    let args = [Value::Number(2.5)];
    assert_eq!(to_percent_fn(&args), Value::Number(2.5));
}

#[test]
fn text_passthrough() {
    let args = [Value::Text("text".to_string())];
    assert_eq!(to_percent_fn(&args), Value::Text("text".to_string()));
}
