use super::super::indirect_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(indirect_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn three_args_returns_na() {
    assert_eq!(
        indirect_fn(&[
            Value::Text("A1".to_string()),
            Value::Bool(true),
            Value::Bool(false),
        ]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn non_text_arg_returns_ref_error() {
    assert_eq!(indirect_fn(&[Value::Number(1.0)]), Value::Error(ErrorKind::Ref));
}
