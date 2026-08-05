use super::super::convert_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args() {
    assert_eq!(convert_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn two_args() {
    assert_eq!(
        convert_fn(&[Value::Number(1.0), Value::Text("km".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn four_args() {
    assert_eq!(
        convert_fn(&[
            Value::Number(1.0),
            Value::Text("km".to_string()),
            Value::Text("m".to_string()),
            Value::Number(0.0),
        ]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn unknown_from_unit() {
    assert_eq!(
        convert_fn(&[Value::Number(1.0), Value::Text("xyz".to_string()), Value::Text("m".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn unknown_to_unit() {
    assert_eq!(
        convert_fn(&[Value::Number(1.0), Value::Text("m".to_string()), Value::Text("xyz".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn incompatible_categories_energy_to_power() {
    assert_eq!(
        convert_fn(&[Value::Number(1.0), Value::Text("J".to_string()), Value::Text("W".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn non_string_from_unit() {
    assert_eq!(
        convert_fn(&[Value::Number(1.0), Value::Number(1.0), Value::Text("m".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn non_string_to_unit() {
    assert_eq!(
        convert_fn(&[Value::Number(1.0), Value::Text("km".to_string()), Value::Bool(true)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn non_numeric_value() {
    assert_eq!(
        convert_fn(&[Value::Text("abc".to_string()), Value::Text("km".to_string()), Value::Text("m".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}
