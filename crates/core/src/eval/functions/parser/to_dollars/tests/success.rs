use super::super::*;
use crate::types::Value;

#[test]
fn integer_passthrough() {
    let args = [Value::Number(42.0)];
    assert_eq!(to_dollars_fn(&args), Value::Number(42.0));
}

#[test]
fn decimal_passthrough() {
    let args = [Value::Number(3.14159)];
    assert_eq!(to_dollars_fn(&args), Value::Number(3.14159));
}

#[test]
fn text_passthrough() {
    let args = [Value::Text("text".to_string())];
    assert_eq!(to_dollars_fn(&args), Value::Text("text".to_string()));
}
