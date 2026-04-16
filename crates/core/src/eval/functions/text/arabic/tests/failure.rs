use super::super::*;
use crate::types::{ErrorKind, Value};

fn run(s: &str) -> Value {
    arabic_fn(&[Value::Text(s.to_string())])
}

#[test]
fn invalid_returns_value_error() {
    assert_eq!(run("ABC"), Value::Error(ErrorKind::Value));
}

#[test]
fn no_args_returns_na() {
    assert_eq!(arabic_fn(&[]), Value::Error(ErrorKind::NA));
}
