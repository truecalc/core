use super::super::*;
use crate::types::Value;

#[test]
fn minus_one_to_oct() {
    assert_eq!(dec2oct_fn(&[Value::Number(-1.0)]), Value::Text("7777777777".to_string()));
}

#[test]
fn eight_to_oct() {
    assert_eq!(dec2oct_fn(&[Value::Number(8.0)]), Value::Text("10".to_string()));
}

#[test]
fn zero_to_oct() {
    assert_eq!(dec2oct_fn(&[Value::Number(0.0)]), Value::Text("0".to_string()));
}

#[test]
fn with_places() {
    assert_eq!(dec2oct_fn(&[Value::Number(8.0), Value::Number(4.0)]), Value::Text("0010".to_string()));
}

#[test]
fn max_positive() {
    assert_eq!(dec2oct_fn(&[Value::Number(536_870_911.0)]), Value::Text("3777777777".to_string()));
}
