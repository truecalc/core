use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn vara_no_args_returns_na() {
    assert_eq!(vara_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn vara_single_value_returns_div_zero() {
    // VARA uses sample variance: n < 2 → DivByZero
    assert_eq!(vara_fn(&[Value::Number(5.0)]), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn vara_only_empty_returns_div_zero() {
    // Empty is skipped; no collected values → n < 2 → DivByZero
    assert_eq!(vara_fn(&[Value::Empty]), Value::Error(ErrorKind::DivByZero));
}
