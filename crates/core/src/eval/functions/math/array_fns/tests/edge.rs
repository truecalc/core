use super::super::{columns_fn, index_fn, rows_fn};
use crate::types::Value;

fn n(v: f64) -> Value {
    Value::Number(v)
}

#[test]
fn rows_of_single_element_2d_array() {
    let arr = Value::Array(vec![Value::Array(vec![n(1.0)])]);
    assert_eq!(rows_fn(&[arr]), n(1.0));
}

#[test]
fn columns_of_single_element_2d_array() {
    let arr = Value::Array(vec![Value::Array(vec![n(1.0)])]);
    assert_eq!(columns_fn(&[arr]), n(1.0));
}

#[test]
fn rows_of_empty_flat_array_returns_1() {
    let arr = Value::Array(vec![]);
    assert_eq!(rows_fn(&[arr]), n(1.0));
}

#[test]
fn index_retrieves_text_from_2d_array() {
    let arr = Value::Array(vec![Value::Array(vec![
        Value::Text("a".into()),
        Value::Text("b".into()),
    ])]);
    assert_eq!(
        index_fn(&[arr, n(1.0), n(2.0)]),
        Value::Text("b".into())
    );
}

#[test]
fn index_retrieves_bool_from_2d_array() {
    let arr = Value::Array(vec![Value::Array(vec![
        Value::Bool(true),
        Value::Bool(false),
    ])]);
    assert_eq!(index_fn(&[arr, n(1.0), n(1.0)]), Value::Bool(true));
}
