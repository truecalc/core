use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn avedev_no_args_returns_na() {
    assert_eq!(avedev_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn avedev_non_numeric_text_returns_value_error() {
    // Direct non-numeric text arg -> #VALUE! (Google Sheets conformant)
    assert_eq!(
        avedev_fn(&[Value::Text("a".to_string()), Value::Number(1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn avedev_empty_only_returns_div_zero() {
    assert_eq!(
        avedev_fn(&[Value::Empty]),
        Value::Error(ErrorKind::DivByZero)
    );
}
