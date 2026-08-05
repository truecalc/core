use super::super::base_fn;
use crate::types::Value;

#[test]
fn base_binary() {
    // BASE(7, 2) = "111"
    assert_eq!(
        base_fn(&[Value::Number(7.0), Value::Number(2.0)]),
        Value::Text("111".to_string())
    );
}

#[test]
fn base_hex() {
    // BASE(255, 16) = "FF"
    assert_eq!(
        base_fn(&[Value::Number(255.0), Value::Number(16.0)]),
        Value::Text("FF".to_string())
    );
}

#[test]
fn base_octal() {
    // BASE(100, 8) = "144"
    assert_eq!(
        base_fn(&[Value::Number(100.0), Value::Number(8.0)]),
        Value::Text("144".to_string())
    );
}

#[test]
fn base_zero() {
    // BASE(0, 16) = "0"
    assert_eq!(
        base_fn(&[Value::Number(0.0), Value::Number(16.0)]),
        Value::Text("0".to_string())
    );
}

#[test]
fn base_with_padding() {
    // BASE(255, 16, 8) = "000000FF"
    assert_eq!(
        base_fn(&[Value::Number(255.0), Value::Number(16.0), Value::Number(8.0)]),
        Value::Text("000000FF".to_string())
    );
}

#[test]
fn base_36_max_digit() {
    // BASE(35, 36) = "Z"
    assert_eq!(
        base_fn(&[Value::Number(35.0), Value::Number(36.0)]),
        Value::Text("Z".to_string())
    );
}

#[test]
fn base_truncates_float() {
    // BASE(7.9, 2) = "111" (truncates to 7)
    assert_eq!(
        base_fn(&[Value::Number(7.9), Value::Number(2.0)]),
        Value::Text("111".to_string())
    );
}
