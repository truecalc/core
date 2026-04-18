use super::super::*;
use crate::types::Value;

#[test]
fn basic_mid() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(1.0), Value::Number(3.0)]),
        Value::Text("Hel".to_string())
    );
}

#[test]
fn mid_from_middle() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(2.0), Value::Number(3.0)]),
        Value::Text("ell".to_string())
    );
}

#[test]
fn mid_with_num_coercion() {
    assert_eq!(
        mid_fn(&[Value::Number(12345.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Text("234".to_string())
    );
}
