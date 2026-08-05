use super::super::split_fn;
use crate::types::Value;

#[test]
fn remove_empty_true_drops_empty_strings() {
    let result = split_fn(&[
        Value::Text("a,,b".into()),
        Value::Text(",".into()),
        Value::Bool(true),
        Value::Bool(true),
    ]);
    assert_eq!(
        result,
        Value::Array(vec![Value::Text("a".into()), Value::Text("b".into())])
    );
}

#[test]
fn remove_empty_false_keeps_empty_as_empty_value() {
    // Empty parts are represented as Value::Empty so COUNTA can correctly skip them
    let result = split_fn(&[
        Value::Text("a,,b".into()),
        Value::Text(",".into()),
        Value::Bool(true),
        Value::Bool(false),
    ]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Text("a".into()),
            Value::Empty,
            Value::Text("b".into()),
        ])
    );
}

#[test]
fn split_by_each_false_treats_delimiter_as_unit() {
    let result = split_fn(&[
        Value::Text("a::b".into()),
        Value::Text("::".into()),
        Value::Bool(false),
    ]);
    assert_eq!(
        result,
        Value::Array(vec![Value::Text("a".into()), Value::Text("b".into())])
    );
}

#[test]
fn default_removes_empty_parts() {
    // 2-arg form: remove_empty defaults to TRUE
    let result = split_fn(&[Value::Text("a,,b".into()), Value::Text(",".into())]);
    assert_eq!(
        result,
        Value::Array(vec![Value::Text("a".into()), Value::Text("b".into())])
    );
}
