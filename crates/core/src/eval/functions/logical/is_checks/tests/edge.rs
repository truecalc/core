use super::super::{isblank_fn, iserror_fn, isna_fn, isnumber_fn, istext_fn};
use crate::types::{ErrorKind, Value};

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
