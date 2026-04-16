use super::super::base_fn;
use crate::types::Value;

#[test]
fn padding_no_op_when_longer() {
    // BASE(255, 16, 2) = "FF" (result already longer than min_length)
    assert_eq!(
        base_fn(&[Value::Number(255.0), Value::Number(16.0), Value::Number(2.0)]),
        Value::Text("FF".to_string())
    );
}

#[test]
fn base_10() {
    // BASE(100, 10) = "100"
    assert_eq!(
        base_fn(&[Value::Number(100.0), Value::Number(10.0)]),
        Value::Text("100".to_string())
    );
}

#[test]
fn base_2_boundary() {
    // BASE(2, 2) = "10"
    assert_eq!(
        base_fn(&[Value::Number(2.0), Value::Number(2.0)]),
        Value::Text("10".to_string())
    );
}
