use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn stdevp_no_args_returns_na() {
    assert_eq!(stdevp_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn stdevp_no_numeric_values_returns_div_zero() {
    assert_eq!(
        stdevp_fn(&[Value::Text("a".to_string()), Value::Bool(false)]),
        Value::Error(ErrorKind::DivByZero)
    );
}
