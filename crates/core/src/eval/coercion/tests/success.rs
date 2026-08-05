use super::super::*;
use crate::types::Value;

#[test]
fn bool_to_number_true() {
    assert_eq!(to_number(Value::Bool(true)), Ok(1.0));
}

#[test]
fn bool_to_number_false() {
    assert_eq!(to_number(Value::Bool(false)), Ok(0.0));
}

#[test]
fn numeric_text_to_number() {
    assert_eq!(to_number(Value::Text("5".into())), Ok(5.0));
}

#[test]
fn empty_to_number() {
    assert_eq!(to_number(Value::Empty), Ok(0.0));
}

#[test]
fn number_to_string() {
    assert_eq!(to_string_val(Value::Number(1.5)), Ok("1.5".to_string()));
    assert_eq!(to_string_val(Value::Number(0.1 + 0.2)), Ok("0.3".to_string()));
}

#[test]
fn bool_to_string_true() {
    assert_eq!(to_string_val(Value::Bool(true)), Ok("TRUE".to_string()));
}

#[test]
fn bool_to_string_false() {
    assert_eq!(to_string_val(Value::Bool(false)), Ok("FALSE".to_string()));
}

#[test]
fn empty_to_string() {
    assert_eq!(to_string_val(Value::Empty), Ok(String::new()));
}

#[test]
fn number_to_bool_nonzero() {
    assert_eq!(to_bool(Value::Number(1.0)), Ok(true));
}

#[test]
fn bool_passthrough() {
    assert_eq!(to_bool(Value::Bool(true)), Ok(true));
    assert_eq!(to_bool(Value::Bool(false)), Ok(false));
}
