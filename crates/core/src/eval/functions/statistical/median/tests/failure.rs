use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn median_no_args_returns_na_error() {
    // MEDIAN() → #N/A (Google Sheets conformance)
    assert_eq!(median_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn median_ignores_error_values() {
    // The dispatcher strips errors before calling median_fn; errors are simply ignored
    assert_eq!(
        median_fn(&[Value::Number(1.0), Value::Error(ErrorKind::Value), Value::Number(3.0)]),
        Value::Number(2.0)
    );
}
