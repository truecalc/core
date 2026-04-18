use super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn inline_array() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]);
    let result = join_fn(&[Value::Text(",".into()), arr]);
    assert_eq!(result, Value::Text("1,2,3".into()));
}

#[test]
fn multiple_scalars() {
    let result = join_fn(&[
        Value::Text("-".into()),
        Value::Text("a".into()),
        Value::Text("b".into()),
        Value::Text("c".into()),
    ]);
    assert_eq!(result, Value::Text("a-b-c".into()));
}

#[test]
fn empty_delimiter() {
    let result = join_fn(&[
        Value::Text("".into()),
        Value::Text("Hello".into()),
        Value::Text(" ".into()),
        Value::Text("World".into()),
    ]);
    assert_eq!(result, Value::Text("Hello World".into()));
}

#[test]
fn multiple_arrays_flattened() {
    let arr1 = Value::Array(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]);
    let arr2 = Value::Array(vec![Value::Number(4.0), Value::Number(5.0), Value::Number(6.0)]);
    let result = join_fn(&[Value::Text(",".into()), arr1, arr2]);
    assert_eq!(result, Value::Text("1,2,3,4,5,6".into()));
}

#[test]
fn single_value() {
    let result = join_fn(&[Value::Text("|".into()), Value::Text("x".into())]);
    assert_eq!(result, Value::Text("x".into()));
}

#[test]
fn number_scalars() {
    let result = join_fn(&[
        Value::Text(",".into()),
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    assert_eq!(result, Value::Text("1,2,3".into()));
}

#[test]
fn no_args_returns_na() {
    assert_eq!(join_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn only_delimiter_returns_na() {
    assert_eq!(join_fn(&[Value::Text(",".into())]), Value::Error(ErrorKind::NA));
}
