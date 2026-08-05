use super::super::*;
use crate::types::Value;

#[test]
fn all_ones_to_hex() {
    assert_eq!(bin2hex_fn(&[Value::Text("1111111111".to_string())]), Value::Text("FFFFFFFFFF".to_string()));
}

#[test]
fn fb() {
    assert_eq!(bin2hex_fn(&[Value::Text("11111011".to_string())]), Value::Text("FB".to_string()));
}

#[test]
fn one_to_hex() {
    assert_eq!(bin2hex_fn(&[Value::Text("1".to_string())]), Value::Text("1".to_string()));
}

#[test]
fn with_places() {
    assert_eq!(bin2hex_fn(&[Value::Text("1111011".to_string()), Value::Number(4.0)]), Value::Text("007B".to_string()));
}

#[test]
fn zero_to_hex() {
    assert_eq!(bin2hex_fn(&[Value::Text("0".to_string())]), Value::Text("0".to_string()));
}
