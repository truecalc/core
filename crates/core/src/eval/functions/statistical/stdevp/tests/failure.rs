use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn stdevp_no_args_returns_na() {
    assert_eq!(stdevp_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn stdevp_direct_text_returns_value_error() {
    // Direct non-parseable text returns #VALUE! (not #DIV/0!)
    assert_eq!(
        stdevp_fn(&[Value::Text("a".to_string()), Value::Bool(false)]),
        Value::Error(ErrorKind::Value)
    );
}
