use super::super::*;
use crate::types::Value;

#[test]
fn basic_lenb() {
    assert_eq!(
        lenb_fn(&[Value::Text("Hello".to_string())]),
        Value::Number(5.0)
    );
}

#[test]
fn number_coerced_to_text() {
    assert_eq!(lenb_fn(&[Value::Number(123.0)]), Value::Number(3.0));
}
