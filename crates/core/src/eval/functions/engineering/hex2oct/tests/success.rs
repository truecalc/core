use super::super::*;
use crate::types::Value;

#[test]
fn f_to_oct() {
    assert_eq!(hex2oct_fn(&[Value::Text("F".to_string())]), Value::Text("17".to_string()));
}

#[test]
fn one_to_oct() {
    assert_eq!(hex2oct_fn(&[Value::Text("1".to_string())]), Value::Text("1".to_string()));
}

#[test]
fn zero_to_oct() {
    assert_eq!(hex2oct_fn(&[Value::Text("0".to_string())]), Value::Text("0".to_string()));
}

#[test]
fn all_f_to_oct() {
    // FFFFFFFFFF = -1 in 40-bit; -1 in 29-bit = 7777777777 octal
    assert_eq!(hex2oct_fn(&[Value::Text("FFFFFFFFFF".to_string())]), Value::Text("7777777777".to_string()));
}

#[test]
fn with_places() {
    assert_eq!(hex2oct_fn(&[Value::Text("1".to_string()), Value::Number(4.0)]), Value::Text("0001".to_string()));
}
