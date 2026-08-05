use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn varp_all_same_values_returns_zero() {
    assert_eq!(
        varp_fn(&[Value::Number(4.0), Value::Number(4.0), Value::Number(4.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn varp_direct_text_returns_value_error() {
    // Direct non-parseable text → #VALUE!
    let result = varp_fn(&[
        Value::Number(2.0),
        Value::Number(6.0),
        Value::Bool(true),
        Value::Text("x".to_string()),
    ]);
    assert_eq!(result, Value::Error(ErrorKind::Value));
}
