use super::super::*;
use crate::types::Value;

#[test]
fn f_to_bin() {
    assert_eq!(hex2bin_fn(&[Value::Text("F".to_string())]), Value::Text("1111".to_string()));
}

#[test]
fn all_f_to_bin() {
    assert_eq!(hex2bin_fn(&[Value::Text("FFFFFFFFFF".to_string())]), Value::Text("1111111111".to_string()));
}

#[test]
fn one_to_bin() {
    assert_eq!(hex2bin_fn(&[Value::Text("1".to_string())]), Value::Text("1".to_string()));
}

#[test]
fn lowercase_hex() {
    assert_eq!(hex2bin_fn(&[Value::Text("ff".to_string())]), Value::Text("11111111".to_string()));
}

#[test]
fn with_places() {
    assert_eq!(hex2bin_fn(&[Value::Text("1".to_string()), Value::Number(4.0)]), Value::Text("0001".to_string()));
}
