use super::super::mina_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(mina_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn all_empty_returns_na() {
    assert_eq!(
        mina_fn(&[Value::Empty, Value::Empty]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn direct_text_arg_returns_value_error() {
    assert_eq!(
        mina_fn(&[Value::Text("text".to_string()), Value::Number(5.0)]),
        Value::Error(ErrorKind::Value)
    );
}
