use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(lenb_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_two_args() {
    assert_eq!(
        lenb_fn(&[Value::Text("Hello".to_string()), Value::Number(2.0)]),
        Value::Error(ErrorKind::NA)
    );
}
