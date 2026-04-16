use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(hex2dec_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn invalid_char_returns_num() {
    assert_eq!(hex2dec_fn(&[Value::Text("G".to_string())]), Value::Error(ErrorKind::Num));
}

#[test]
fn too_long_returns_num() {
    assert_eq!(hex2dec_fn(&[Value::Text("FFFFFFFFFFF".to_string())]), Value::Error(ErrorKind::Num));
}
