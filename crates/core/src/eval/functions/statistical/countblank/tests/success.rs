use super::super::count_blanks_in;
use crate::types::Value;

fn cb(args: &[Value]) -> Value {
    Value::Number(count_blanks_in(args) as f64)
}

#[test]
fn empty_string_is_blank() {
    assert_eq!(cb(&[Value::Text("".into())]), Value::Number(1.0));
}

#[test]
fn non_empty_text_not_blank() {
    assert_eq!(cb(&[Value::Text("hello".into())]), Value::Number(0.0));
}

#[test]
fn number_not_blank() {
    assert_eq!(cb(&[Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn bool_false_not_blank() {
    assert_eq!(cb(&[Value::Bool(false)]), Value::Number(0.0));
}

#[test]
fn bool_true_not_blank() {
    assert_eq!(cb(&[Value::Bool(true)]), Value::Number(0.0));
}

#[test]
fn array_counts_empty_strings() {
    let arr = Value::Array(vec![
        Value::Text("".into()),
        Value::Number(1.0),
        Value::Text("".into()),
        Value::Number(2.0),
    ]);
    assert_eq!(cb(&[arr]), Value::Number(2.0));
}

#[test]
fn array_all_blank() {
    let arr = Value::Array(vec![
        Value::Text("".into()),
        Value::Text("".into()),
        Value::Text("".into()),
    ]);
    assert_eq!(cb(&[arr]), Value::Number(3.0));
}

#[test]
fn array_no_blanks() {
    let arr = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    assert_eq!(cb(&[arr]), Value::Number(0.0));
}
