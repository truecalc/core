use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn max_no_args_returns_na() {
    assert_eq!(max_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn max_text_in_args_returns_value_error() {
    assert_eq!(
        max_fn(&[Value::Text("a".to_string()), Value::Bool(true), Value::Empty]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn max_empty_array_is_ref_error() {
    assert_eq!(max_fn(&[Value::Array(vec![])]), Value::Error(ErrorKind::Ref));
}

#[test]
fn max_non_empty_array_without_numbers_returns_zero() {
    // `=MAX({"a","b"})` and `=MAX({TRUE,FALSE})` are both 0: neither text nor
    // booleans contribute a number in array context, but they do make the
    // argument present rather than absent.
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
        ])]),
        Value::Number(0.0)
    );
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Bool(true), Value::Bool(false)])]),
        Value::Number(0.0)
    );
}

#[test]
fn max_array_of_only_blanks_is_ref_error() {
    // `=MAX(<range of blank cells>)` stays #REF! — this is the half the
    // narrowing above must not disturb.
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Empty, Value::Empty, Value::Empty])]),
        Value::Error(ErrorKind::Ref)
    );
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Array(vec![Value::Empty]),
            Value::Array(vec![Value::Empty]),
        ])]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn max_blanks_beside_content_are_not_absent() {
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Empty,
            Value::Text("z".to_string()),
        ])]),
        Value::Number(0.0)
    );
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Empty, Value::Number(4.0)])]),
        Value::Number(4.0)
    );
}

#[test]
fn max_negative_numbers() {
    assert_eq!(
        max_fn(&[Value::Number(-3.0), Value::Number(-1.0), Value::Number(-5.0)]),
        Value::Number(-1.0)
    );
}
