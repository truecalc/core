use super::super::*;
use crate::types::Value;

#[test]
fn bool_coerced() {
    assert_eq!(
        len_fn(&[Value::Bool(true)]),
        Value::Number(4.0) // "TRUE"
    );
}

#[test]
fn empty_value() {
    assert_eq!(
        len_fn(&[Value::Empty]),
        Value::Number(0.0)
    );
}

#[test]
fn spaces_count() {
    assert_eq!(
        len_fn(&[Value::Text("  a  ".to_string())]),
        Value::Number(5.0)
    );
}
