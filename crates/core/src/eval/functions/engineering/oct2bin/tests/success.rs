use super::super::*;
use crate::types::Value;

#[test]
fn seven_to_bin() {
    assert_eq!(oct2bin_fn(&[Value::Text("7".to_string())]), Value::Text("111".to_string()));
}

#[test]
fn one_to_bin() {
    assert_eq!(oct2bin_fn(&[Value::Text("1".to_string())]), Value::Text("1".to_string()));
}

#[test]
fn zero_to_bin() {
    assert_eq!(oct2bin_fn(&[Value::Text("0".to_string())]), Value::Text("0".to_string()));
}

#[test]
fn all_sevens_to_bin() {
    // 7777777777 octal = -1 (29-bit two's complement), which gives 1111111111 in binary
    assert_eq!(oct2bin_fn(&[Value::Text("7777777777".to_string())]), Value::Text("1111111111".to_string()));
}

#[test]
fn with_places() {
    assert_eq!(oct2bin_fn(&[Value::Text("3".to_string()), Value::Number(4.0)]), Value::Text("0011".to_string()));
}
