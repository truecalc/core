use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn average_negatives() {
    assert_eq!(
        average_fn(&[Value::Number(-3.0), Value::Number(-1.0)]),
        Value::Number(-2.0)
    );
}

#[test]
fn average_empty_skipped() {
    // Empty is skipped; only Number(4.0) counted → average = 4.0
    assert_eq!(
        average_fn(&[Value::Empty, Value::Number(4.0)]),
        Value::Number(4.0)
    );
}

#[test]
fn overflow_returns_num_error() {
    assert_eq!(
        average_fn(&[Value::Number(f64::MAX), Value::Number(f64::MAX)]),
        Value::Error(ErrorKind::Num)
    );
}
