use super::super::decimal_fn;
use crate::types::Value;

#[test]
fn base_10_passthrough() {
    // DECIMAL("100", 10) = 100
    assert_eq!(
        decimal_fn(&[Value::Text("100".to_string()), Value::Number(10.0)]),
        Value::Number(100.0)
    );
}

#[test]
fn single_digit() {
    // DECIMAL("1", 2) = 1
    assert_eq!(
        decimal_fn(&[Value::Text("1".to_string()), Value::Number(2.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn lowercase_z_base_36() {
    // DECIMAL("z", 36) = 35 (case-insensitive)
    assert_eq!(
        decimal_fn(&[Value::Text("z".to_string()), Value::Number(36.0)]),
        Value::Number(35.0)
    );
}
