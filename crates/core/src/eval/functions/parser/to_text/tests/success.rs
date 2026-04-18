use super::super::*;
use crate::types::Value;

#[test]
fn integer_to_text() {
    let args = [Value::Number(42.0)];
    assert_eq!(to_text_fn(&args), Value::Text("42".to_string()));
}

#[test]
fn decimal_to_text() {
    let args = [Value::Number(3.14)];
    assert_eq!(to_text_fn(&args), Value::Text("3.14".to_string()));
}

#[test]
fn bool_true_to_text() {
    let args = [Value::Bool(true)];
    assert_eq!(to_text_fn(&args), Value::Text("TRUE".to_string()));
}

#[test]
fn bool_false_to_text() {
    let args = [Value::Bool(false)];
    assert_eq!(to_text_fn(&args), Value::Text("FALSE".to_string()));
}

#[test]
fn text_passthrough() {
    let args = [Value::Text("hello".to_string())];
    assert_eq!(to_text_fn(&args), Value::Text("hello".to_string()));
}

#[test]
fn large_integer_to_text() {
    let args = [Value::Number(100000.0)];
    assert_eq!(to_text_fn(&args), Value::Text("100000".to_string()));
}

#[test]
fn small_decimal_to_text() {
    let args = [Value::Number(0.001)];
    assert_eq!(to_text_fn(&args), Value::Text("0.001".to_string()));
}
