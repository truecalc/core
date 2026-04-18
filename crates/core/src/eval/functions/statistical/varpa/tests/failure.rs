use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn varpa_no_args_returns_na() {
    assert_eq!(varpa_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn varpa_only_empty_returns_div_zero() {
    // Empty is skipped; no collected values → n == 0 → DivByZero
    assert_eq!(varpa_fn(&[Value::Empty]), Value::Error(ErrorKind::DivByZero));
}
