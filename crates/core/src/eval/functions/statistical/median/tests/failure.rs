use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn median_no_args_returns_na() {
    // MEDIAN() → #N/A (Google Sheets)
    assert_eq!(median_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn median_propagates_error_values() {
    // Direct errors propagate through median_fn
    assert_eq!(
        median_fn(&[Value::Number(1.0), Value::Error(ErrorKind::Value), Value::Number(3.0)]),
        Value::Error(ErrorKind::Value)
    );
}
