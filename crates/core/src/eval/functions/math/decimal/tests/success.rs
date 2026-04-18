use super::super::decimal_fn;
use crate::types::Value;

#[test]
fn hex_to_decimal() {
    // DECIMAL("FF", 16) = 255
    assert_eq!(
        decimal_fn(&[Value::Text("FF".to_string()), Value::Number(16.0)]),
        Value::Number(255.0)
    );
}

#[test]
fn binary_to_decimal() {
    // DECIMAL("111", 2) = 7
    assert_eq!(
        decimal_fn(&[Value::Text("111".to_string()), Value::Number(2.0)]),
        Value::Number(7.0)
    );
}

#[test]
fn octal_to_decimal() {
    // DECIMAL("144", 8) = 100
    assert_eq!(
        decimal_fn(&[Value::Text("144".to_string()), Value::Number(8.0)]),
        Value::Number(100.0)
    );
}

#[test]
fn zero() {
    // DECIMAL("0", 16) = 0
    assert_eq!(
        decimal_fn(&[Value::Text("0".to_string()), Value::Number(16.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn case_insensitive() {
    // DECIMAL("ff", 16) = 255
    assert_eq!(
        decimal_fn(&[Value::Text("ff".to_string()), Value::Number(16.0)]),
        Value::Number(255.0)
    );
}

#[test]
fn base_36() {
    // DECIMAL("Z", 36) = 35
    assert_eq!(
        decimal_fn(&[Value::Text("Z".to_string()), Value::Number(36.0)]),
        Value::Number(35.0)
    );
}
