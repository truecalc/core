use super::super::averagea_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(averagea_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn all_empty_returns_div_zero() {
    assert_eq!(
        averagea_fn(&[Value::Empty, Value::Empty]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn direct_error_propagates() {
    assert_eq!(
        averagea_fn(&[Value::Error(crate::types::ErrorKind::Value)]),
        Value::Error(crate::types::ErrorKind::Value)
    );
}
