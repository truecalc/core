use super::super::*;
use crate::types::Value;

#[test]
fn minus_one_to_bin() {
    assert_eq!(dec2bin_fn(&[Value::Number(-1.0)]), Value::Text("1111111111".to_string()));
}

#[test]
fn max_positive() {
    assert_eq!(dec2bin_fn(&[Value::Number(511.0)]), Value::Text("111111111".to_string()));
}

#[test]
fn min_negative() {
    assert_eq!(dec2bin_fn(&[Value::Number(-512.0)]), Value::Text("1000000000".to_string()));
}

#[test]
fn one_to_bin() {
    assert_eq!(dec2bin_fn(&[Value::Number(1.0)]), Value::Text("1".to_string()));
}

#[test]
fn zero_to_bin() {
    assert_eq!(dec2bin_fn(&[Value::Number(0.0)]), Value::Text("0".to_string()));
}

#[test]
fn with_places() {
    assert_eq!(dec2bin_fn(&[Value::Number(9.0), Value::Number(4.0)]), Value::Text("1001".to_string()));
}

#[test]
fn truncates_decimal() {
    assert_eq!(dec2bin_fn(&[Value::Number(9.9)]), Value::Text("1001".to_string()));
}
