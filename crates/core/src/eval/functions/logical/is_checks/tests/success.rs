use super::super::{is_valid_email, isblank_fn, iserror_fn, isna_fn, isnumber_fn, istext_fn};
use crate::types::{ErrorKind, Value};

#[test]
fn isnumber_with_number() {
    assert_eq!(isnumber_fn(&[Value::Number(3.14)]), Value::Bool(true));
}

#[test]
fn istext_with_text() {
    assert_eq!(istext_fn(&[Value::Text("hello".to_string())]), Value::Bool(true));
}

#[test]
fn iserror_with_error() {
    assert_eq!(iserror_fn(&[Value::Error(ErrorKind::DivByZero)]), Value::Bool(true));
}

#[test]
fn isblank_with_empty() {
    assert_eq!(isblank_fn(&[Value::Empty]), Value::Bool(true));
}

#[test]
fn isna_with_na() {
    assert_eq!(isna_fn(&[Value::Error(ErrorKind::NA)]), Value::Bool(true));
}

#[test]
fn isemail_valid_simple() {
    assert!(is_valid_email("user@example.com"));
}

#[test]
fn isemail_valid_subdomain() {
    assert!(is_valid_email("user@mail.example.com"));
}

#[test]
fn isemail_valid_plus_tag() {
    assert!(is_valid_email("user+tag@example.com"));
}
