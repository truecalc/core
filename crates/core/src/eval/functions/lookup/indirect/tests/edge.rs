use super::super::indirect_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn empty_string_returns_ref_error() {
    assert_eq!(indirect_fn(&[Value::Text("".to_string())]), Value::Error(ErrorKind::Ref));
}

#[test]
fn invalid_ref_returns_ref_error() {
    assert_eq!(
        indirect_fn(&[Value::Text("InvalidRef!".to_string())]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn r1c1_false_mode_returns_empty() {
    assert_eq!(
        indirect_fn(&[Value::Text("A1".to_string()), Value::Bool(false)]),
        Value::Empty
    );
}
