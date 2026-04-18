use super::super::gcd_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args() {
    assert_eq!(gcd_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn negative_value() {
    assert_eq!(
        gcd_fn(&[Value::Number(-1.0), Value::Number(5.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn non_numeric_arg() {
    assert_eq!(
        gcd_fn(&[Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}
