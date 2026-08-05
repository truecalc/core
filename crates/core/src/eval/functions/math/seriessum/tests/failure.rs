use super::super::seriessum_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(seriessum_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_three_args() {
    assert_eq!(
        seriessum_fn(&[Value::Number(1.0), Value::Number(0.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}
