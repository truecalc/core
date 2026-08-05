use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn mode_all_unique_returns_na() {
    assert_eq!(
        mode_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn mode_no_values_returns_na() {
    assert_eq!(mode_fn(&[]), Value::Error(ErrorKind::NA));
}
