use super::super::maxifs_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(maxifs_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn two_args_returns_na() {
    assert_eq!(
        maxifs_fn(&[Value::Number(1.0), Value::Number(2.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn array_literal_as_max_range_returns_na() {
    // GS requires cell ranges, not inline array literals → #N/A
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    let crit = Value::Text(">1".to_string());
    assert_eq!(
        maxifs_fn(&[arr.clone(), arr.clone(), crit]),
        Value::Error(ErrorKind::NA)
    );
}
