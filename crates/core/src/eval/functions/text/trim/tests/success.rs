use super::super::*;
use crate::types::Value;

#[test]
fn basic_trim() {
    assert_eq!(
        trim_fn(&[Value::Text("  Hello  ".to_string())]),
        Value::Text("Hello".to_string())
    );
}

#[test]
fn internal_spaces_collapsed() {
    assert_eq!(
        trim_fn(&[Value::Text("  a  b  ".to_string())]),
        Value::Text("a b".to_string())
    );
}

#[test]
fn no_change_needed() {
    assert_eq!(
        trim_fn(&[Value::Text("Hello World".to_string())]),
        Value::Text("Hello World".to_string())
    );
}
