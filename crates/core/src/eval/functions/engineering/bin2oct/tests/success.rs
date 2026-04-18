use super::super::*;
use crate::types::Value;

#[test]
fn all_ones_to_oct() {
    assert_eq!(bin2oct_fn(&[Value::Text("1111111111".to_string())]), Value::Text("7777777777".to_string()));
}

#[test]
fn one_to_oct() {
    assert_eq!(bin2oct_fn(&[Value::Text("1".to_string())]), Value::Text("1".to_string()));
}

#[test]
fn zero_to_oct() {
    assert_eq!(bin2oct_fn(&[Value::Text("0".to_string())]), Value::Text("0".to_string()));
}

#[test]
fn with_places() {
    assert_eq!(bin2oct_fn(&[Value::Text("1000".to_string()), Value::Number(4.0)]), Value::Text("0010".to_string()));
}

#[test]
fn eight_to_oct() {
    assert_eq!(bin2oct_fn(&[Value::Text("1000".to_string())]), Value::Text("10".to_string()));
}
