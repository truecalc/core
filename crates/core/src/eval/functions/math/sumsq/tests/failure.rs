use super::super::sumsq_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(sumsq_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn error_propagated() {
    let result = sumsq_fn(&[Value::Number(1.0), Value::Error(ErrorKind::Value)]);
    assert_eq!(result, Value::Error(ErrorKind::Value));
}
