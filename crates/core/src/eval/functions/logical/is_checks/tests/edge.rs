use super::super::{is_valid_email, isblank_fn, iserror_fn, isna_fn, isnumber_fn, istext_fn};
use crate::evaluate;
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

#[test]
fn isnumber_false_for_bool() {
    assert_eq!(isnumber_fn(&[Value::Bool(true)]), Value::Bool(false));
}

#[test]
fn istext_false_for_number() {
    assert_eq!(istext_fn(&[Value::Number(1.0)]), Value::Bool(false));
}

#[test]
fn iserror_false_for_non_error() {
    assert_eq!(iserror_fn(&[Value::Number(1.0)]), Value::Bool(false));
}

#[test]
fn isblank_false_for_text() {
    assert_eq!(isblank_fn(&[Value::Text(String::new())]), Value::Bool(false));
}

/// ISNA returns false for other error kinds (not just NA).
#[test]
fn isna_false_for_other_error() {
    assert_eq!(isna_fn(&[Value::Error(ErrorKind::Value)]), Value::Bool(false));
}

#[test]
fn isemail_missing_at_sign() {
    assert!(!is_valid_email("notanemail"));
}

#[test]
fn isemail_missing_domain() {
    assert!(!is_valid_email("user@"));
}

#[test]
fn isemail_missing_local_part() {
    assert!(!is_valid_email("@example.com"));
}

#[test]
fn isemail_no_dot_in_domain() {
    assert!(!is_valid_email("user@localhost"));
}

#[test]
fn isemail_number_arg_returns_false() {
    assert_eq!(evaluate("=ISEMAIL(42)", &HashMap::new()), Value::Bool(false));
}

#[test]
fn isemail_bool_arg_returns_false() {
    assert_eq!(evaluate("=ISEMAIL(TRUE)", &HashMap::new()), Value::Bool(false));
}
