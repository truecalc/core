use super::super::countblank_fn;
use crate::types::Value;

#[test]
fn empty_string_is_blank() {
    let r = countblank_fn(&[Value::Text("".into())]);
    assert_eq!(r, Value::Number(1.0));
}

#[test]
fn non_empty_text_not_blank() {
    let r = countblank_fn(&[Value::Text("hello".into())]);
    assert_eq!(r, Value::Number(0.0));
}

#[test]
fn number_not_blank() {
    let r = countblank_fn(&[Value::Number(0.0)]);
    assert_eq!(r, Value::Number(0.0));
}

#[test]
fn bool_false_not_blank() {
    let r = countblank_fn(&[Value::Bool(false)]);
    assert_eq!(r, Value::Number(0.0));
}

#[test]
fn bool_true_not_blank() {
    let r = countblank_fn(&[Value::Bool(true)]);
    assert_eq!(r, Value::Number(0.0));
}

#[test]
fn array_counts_empty_strings() {
    let arr = Value::Array(vec![
        Value::Text("".into()),
        Value::Number(1.0),
        Value::Text("".into()),
        Value::Number(2.0),
    ]);
    let r = countblank_fn(&[arr]);
    assert_eq!(r, Value::Number(2.0));
}

#[test]
fn array_all_blank() {
    let arr = Value::Array(vec![
        Value::Text("".into()),
        Value::Text("".into()),
        Value::Text("".into()),
    ]);
    let r = countblank_fn(&[arr]);
    assert_eq!(r, Value::Number(3.0));
}

#[test]
fn array_no_blanks() {
    let arr = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    let r = countblank_fn(&[arr]);
    assert_eq!(r, Value::Number(0.0));
}
