use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(bin2dec_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn invalid_char_returns_num() {
    assert_eq!(bin2dec_fn(&[Value::Text("2".to_string())]), Value::Error(ErrorKind::Num));
}

#[test]
fn too_long_returns_num() {
    assert_eq!(bin2dec_fn(&[Value::Text("11111111111".to_string())]), Value::Error(ErrorKind::Num));
}

#[test]
fn non_binary_string_returns_num() {
    assert_eq!(bin2dec_fn(&[Value::Text("ABC".to_string())]), Value::Error(ErrorKind::Num));
}
