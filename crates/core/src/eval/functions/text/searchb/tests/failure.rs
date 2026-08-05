use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn not_found() {
    assert_eq!(
        searchb_fn(&[
            Value::Text("z".to_string()),
            Value::Text("Hello".to_string())
        ]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(searchb_fn(&[]), Value::Error(ErrorKind::NA));
}
