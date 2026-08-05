use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn stdevpa_no_args_returns_na() {
    assert_eq!(stdevpa_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn stdevpa_only_empty_returns_div_zero() {
    // Empty is skipped; n == 0 → DivByZero
    assert_eq!(stdevpa_fn(&[Value::Empty]), Value::Error(ErrorKind::DivByZero));
}
