use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn devsq_no_args_returns_na() {
    assert_eq!(devsq_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn devsq_non_numeric_text_returns_value_error() {
    // Direct non-numeric text -> #VALUE! (Google Sheets conformant)
    assert_eq!(
        devsq_fn(&[Value::Text("a".to_string()), Value::Number(1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn devsq_empty_only_returns_num_error() {
    assert_eq!(devsq_fn(&[Value::Empty]), Value::Error(ErrorKind::Num));
}
