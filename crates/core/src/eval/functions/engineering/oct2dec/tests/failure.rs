use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(oct2dec_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn invalid_char_returns_num() {
    assert_eq!(oct2dec_fn(&[Value::Text("8".to_string())]), Value::Error(ErrorKind::Num));
}

#[test]
fn too_long_returns_num() {
    assert_eq!(oct2dec_fn(&[Value::Text("77777777777".to_string())]), Value::Error(ErrorKind::Num));
}
