use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(oct2bin_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn invalid_char_returns_num() {
    assert_eq!(oct2bin_fn(&[Value::Text("8".to_string())]), Value::Error(ErrorKind::Num));
}

#[test]
fn out_of_bin_range_returns_num() {
    // 1000 octal = 512 decimal, out of 10-bit signed range
    assert_eq!(oct2bin_fn(&[Value::Text("1000".to_string())]), Value::Error(ErrorKind::Num));
}

#[test]
fn places_zero_returns_num() {
    assert_eq!(oct2bin_fn(&[Value::Text("1".to_string()), Value::Number(0.0)]), Value::Error(ErrorKind::Num));
}
