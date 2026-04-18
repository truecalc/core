use super::super::combina_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args() {
    assert_eq!(combina_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn one_arg() {
    assert_eq!(combina_fn(&[Value::Number(5.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn three_args() {
    assert_eq!(
        combina_fn(&[Value::Number(5.0), Value::Number(2.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn negative_n() {
    assert_eq!(combina_fn(&[Value::Number(-1.0), Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn negative_k() {
    assert_eq!(combina_fn(&[Value::Number(5.0), Value::Number(-1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn non_numeric_arg() {
    assert_eq!(
        combina_fn(&[Value::Text("abc".to_string()), Value::Number(2.0)]),
        Value::Error(ErrorKind::Value)
    );
}
