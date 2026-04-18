use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn non_numeric_text_to_number() {
    assert_eq!(to_number(Value::Text("abc".into())), Err(Value::Error(ErrorKind::Value)));
}

#[test]
fn error_propagates_through_to_number() {
    let err = Value::Error(ErrorKind::DivByZero);
    assert_eq!(to_number(err.clone()), Err(err));
}

#[test]
fn non_bool_text_to_bool_fails() {
    assert_eq!(to_bool(Value::Text("abc".into())), Err(Value::Error(ErrorKind::Value)));
}

#[test]
fn true_text_to_bool_succeeds() {
    assert_eq!(to_bool(Value::Text("true".into())), Ok(true));
}

#[test]
fn false_text_to_bool_succeeds() {
    assert_eq!(to_bool(Value::Text("FALSE".into())), Ok(false));
}

#[test]
fn empty_to_bool_fails() {
    assert_eq!(to_bool(Value::Empty), Err(Value::Error(ErrorKind::Value)));
}

#[test]
fn error_propagates_through_to_bool() {
    let err = Value::Error(ErrorKind::Ref);
    assert_eq!(to_bool(err.clone()), Err(err));
}

#[test]
fn error_propagates_through_to_string_val() {
    let err = Value::Error(ErrorKind::Name);
    assert_eq!(to_string_val(err.clone()), Err(err));
}
