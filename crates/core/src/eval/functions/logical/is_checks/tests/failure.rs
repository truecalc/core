use super::super::{isblank_fn, iserror_fn, isna_fn, isnumber_fn, istext_fn};
use crate::types::{ErrorKind, Value};

#[test]
fn isnumber_too_many_args() {
    assert_eq!(
        isnumber_fn(&[Value::Number(1.0), Value::Number(2.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn istext_no_args() {
    assert_eq!(istext_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn iserror_too_many_args() {
    assert_eq!(
        iserror_fn(&[Value::Bool(true), Value::Bool(false)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn isblank_no_args() {
    assert_eq!(isblank_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn isna_no_args() {
    assert_eq!(isna_fn(&[]), Value::Error(ErrorKind::Value));
}
