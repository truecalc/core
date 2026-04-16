use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn mode_single_value() {
    // MODE(7) — only one value; all unique, returns #N/A
    assert_eq!(mode_fn(&[Value::Number(7.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn mode_negative_values() {
    // MODE(-3, -1, -3) = -3
    assert_eq!(
        mode_fn(&[
            Value::Number(-3.0),
            Value::Number(-1.0),
            Value::Number(-3.0)
        ]),
        Value::Number(-3.0)
    );
}

#[test]
fn mode_ignores_non_numeric() {
    // Text and Bool are ignored; numeric set {2,2,3} → mode 2
    assert_eq!(
        mode_fn(&[
            Value::Number(2.0),
            Value::Text("hello".to_string()),
            Value::Bool(true),
            Value::Number(2.0),
            Value::Number(3.0)
        ]),
        Value::Number(2.0)
    );
}

#[test]
fn mode_array_arg() {
    // MODE via Array argument: [1, 2, 2, 3] → 2
    let arr = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    assert_eq!(mode_fn(&[arr]), Value::Number(2.0));
}
