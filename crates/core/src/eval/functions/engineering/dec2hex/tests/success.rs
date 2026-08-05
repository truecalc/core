use super::super::*;
use crate::types::Value;

#[test]
fn minus_one_to_hex() {
    assert_eq!(dec2hex_fn(&[Value::Number(-1.0)]), Value::Text("FFFFFFFFFF".to_string()));
}

#[test]
fn two_fifty_five() {
    assert_eq!(dec2hex_fn(&[Value::Number(255.0)]), Value::Text("FF".to_string()));
}

#[test]
fn zero_to_hex() {
    assert_eq!(dec2hex_fn(&[Value::Number(0.0)]), Value::Text("0".to_string()));
}

#[test]
fn with_places() {
    assert_eq!(dec2hex_fn(&[Value::Number(255.0), Value::Number(4.0)]), Value::Text("00FF".to_string()));
}

#[test]
fn max_positive() {
    assert_eq!(dec2hex_fn(&[Value::Number(549_755_813_887.0)]), Value::Text("7FFFFFFFFF".to_string()));
}
