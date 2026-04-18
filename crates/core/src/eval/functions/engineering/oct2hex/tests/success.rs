use super::super::*;
use crate::types::Value;

#[test]
fn seven_to_hex() {
    assert_eq!(oct2hex_fn(&[Value::Text("7".to_string())]), Value::Text("7".to_string()));
}

#[test]
fn ten_octal_to_hex() {
    assert_eq!(oct2hex_fn(&[Value::Text("10".to_string())]), Value::Text("8".to_string()));
}

#[test]
fn all_sevens_to_hex() {
    // 7777777777 = -1, which is FFFFFFFFFF in hex
    assert_eq!(oct2hex_fn(&[Value::Text("7777777777".to_string())]), Value::Text("FFFFFFFFFF".to_string()));
}

#[test]
fn with_places() {
    assert_eq!(oct2hex_fn(&[Value::Text("7".to_string()), Value::Number(2.0)]), Value::Text("07".to_string()));
}

#[test]
fn zero_to_hex() {
    assert_eq!(oct2hex_fn(&[Value::Text("0".to_string())]), Value::Text("0".to_string()));
}
