use super::super::count_blanks_in;
use crate::types::Value;

fn cb(args: &[Value]) -> Value {
    Value::Number(count_blanks_in(args) as f64)
}

#[test]
fn array_with_empty_values_counted() {
    let arr = Value::Array(vec![
        Value::Empty,
        Value::Text("a".to_string()),
        Value::Empty,
        Value::Text("b".to_string()),
    ]);
    assert_eq!(cb(&[arr]), Value::Number(2.0));
}

#[test]
fn array_mixed_empty_and_blank_strings() {
    let arr = Value::Array(vec![
        Value::Empty,
        Value::Text("".to_string()),
        Value::Number(1.0),
        Value::Text("hello".to_string()),
    ]);
    assert_eq!(cb(&[arr]), Value::Number(2.0));
}

#[test]
fn array_mixed_types_no_blanks() {
    let arr = Value::Array(vec![
        Value::Number(1.0),
        Value::Text("a".to_string()),
        Value::Bool(true),
    ]);
    assert_eq!(cb(&[arr]), Value::Number(0.0));
}
