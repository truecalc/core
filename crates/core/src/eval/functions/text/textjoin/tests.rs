use super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn basic_join_ignore_empty_true() {
    let result = textjoin_fn(&[
        Value::Text(",".into()),
        Value::Bool(true),
        Value::Text("a".into()),
        Value::Text("b".into()),
        Value::Text("c".into()),
    ]);
    assert_eq!(result, Value::Text("a,b,c".into()));
}

#[test]
fn skip_empty_when_ignore_true() {
    let result = textjoin_fn(&[
        Value::Text(",".into()),
        Value::Bool(true),
        Value::Text("a".into()),
        Value::Text("".into()),
        Value::Text("c".into()),
    ]);
    assert_eq!(result, Value::Text("a,c".into()));
}

#[test]
fn keep_empty_when_ignore_false() {
    let result = textjoin_fn(&[
        Value::Text(",".into()),
        Value::Bool(false),
        Value::Text("a".into()),
        Value::Text("".into()),
        Value::Text("c".into()),
    ]);
    assert_eq!(result, Value::Text("a,,c".into()));
}

#[test]
fn array_arg() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]);
    let result = textjoin_fn(&[Value::Text("-".into()), Value::Bool(true), arr]);
    assert_eq!(result, Value::Text("1-2-3".into()));
}

#[test]
fn empty_delimiter() {
    let result = textjoin_fn(&[
        Value::Text("".into()),
        Value::Bool(true),
        Value::Text("Hello".into()),
        Value::Text(" ".into()),
        Value::Text("World".into()),
    ]);
    assert_eq!(result, Value::Text("Hello World".into()));
}

#[test]
fn single_value() {
    let result = textjoin_fn(&[
        Value::Text(",".into()),
        Value::Bool(true),
        Value::Text("only".into()),
    ]);
    assert_eq!(result, Value::Text("only".into()));
}

#[test]
fn all_empty_ignored() {
    let result = textjoin_fn(&[
        Value::Text(",".into()),
        Value::Bool(true),
        Value::Text("".into()),
        Value::Text("".into()),
        Value::Text("".into()),
    ]);
    assert_eq!(result, Value::Text("".into()));
}

#[test]
fn pipe_delimiter() {
    let result = textjoin_fn(&[
        Value::Text("|".into()),
        Value::Bool(true),
        Value::Text("x".into()),
        Value::Text("y".into()),
        Value::Text("z".into()),
    ]);
    assert_eq!(result, Value::Text("x|y|z".into()));
}

#[test]
fn no_args_returns_na() {
    assert_eq!(textjoin_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_few_args_returns_na() {
    let r = textjoin_fn(&[Value::Text(",".into()), Value::Bool(true)]);
    assert_eq!(r, Value::Error(ErrorKind::NA));
}
