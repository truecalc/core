use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn devsq_no_args_returns_na() {
    assert_eq!(devsq_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn devsq_no_numeric_values_returns_num_error() {
    assert_eq!(
        devsq_fn(&[Value::Text("a".to_string()), Value::Bool(false)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn devsq_empty_only_returns_num_error() {
    assert_eq!(devsq_fn(&[Value::Empty]), Value::Error(ErrorKind::Num));
}
