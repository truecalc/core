use crate::Engine;
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

#[test]
fn zero_rows_returns_num_error() {
    let eng = Engine::sheets();
    assert_eq!(eng.evaluate("=RANDARRAY(0)", &HashMap::new()), Value::Error(ErrorKind::Num));
}

#[test]
fn negative_rows_returns_num_error() {
    let eng = Engine::sheets();
    assert_eq!(eng.evaluate("=RANDARRAY(-1)", &HashMap::new()), Value::Error(ErrorKind::Num));
}

#[test]
fn zero_cols_returns_num_error() {
    let eng = Engine::sheets();
    assert_eq!(eng.evaluate("=RANDARRAY(2, 0)", &HashMap::new()), Value::Error(ErrorKind::Num));
}

#[test]
fn negative_cols_returns_num_error() {
    let eng = Engine::sheets();
    assert_eq!(eng.evaluate("=RANDARRAY(2, -3)", &HashMap::new()), Value::Error(ErrorKind::Num));
}

#[test]
fn too_many_args_returns_na() {
    let eng = Engine::sheets();
    // 6 arguments exceed the max of 5
    assert_eq!(
        eng.evaluate("=RANDARRAY(1, 1, 0, 1, TRUE, EXTRA)", &HashMap::new()),
        Value::Error(ErrorKind::NA)
    );
}
