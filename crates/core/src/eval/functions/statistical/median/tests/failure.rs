use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn median_no_numeric_returns_num_error() {
    // MEDIAN() → #NUM!
    assert_eq!(median_fn(&[]), Value::Error(ErrorKind::Num));
}

#[test]
fn median_ignores_error_values() {
    // The dispatcher strips errors before calling median_fn; errors are simply ignored
    assert_eq!(
        median_fn(&[Value::Number(1.0), Value::Error(ErrorKind::Value), Value::Number(3.0)]),
        Value::Number(2.0)
    );
}
