use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn not_found() {
    assert_eq!(
        findb_fn(&[
            Value::Text("z".to_string()),
            Value::Text("Hello".to_string())
        ]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(findb_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn case_sensitive_no_match() {
    assert_eq!(
        findb_fn(&[
            Value::Text("hello".to_string()),
            Value::Text("Hello".to_string())
        ]),
        Value::Error(ErrorKind::Value)
    );
}
