use crate::Engine;
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

#[test]
fn rand_with_args_returns_value_error() {
    let eng = Engine::sheets();
    assert_eq!(eng.evaluate("=RAND(1)", &HashMap::new()), Value::Error(ErrorKind::NA));
}

#[test]
fn randbetween_no_args_returns_value_error() {
    let eng = Engine::sheets();
    assert_eq!(eng.evaluate("=RANDBETWEEN()", &HashMap::new()), Value::Error(ErrorKind::NA));
}

#[test]
fn randbetween_low_greater_than_high_returns_num_error() {
    let eng = Engine::sheets();
    assert_eq!(eng.evaluate("=RANDBETWEEN(10, 1)", &HashMap::new()), Value::Error(ErrorKind::Num));
}
