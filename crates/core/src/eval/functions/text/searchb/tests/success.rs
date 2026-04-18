use super::super::*;
use crate::types::Value;

#[test]
fn case_insensitive_search() {
    assert_eq!(
        searchb_fn(&[
            Value::Text("hello".to_string()),
            Value::Text("Hello World".to_string())
        ]),
        Value::Number(1.0)
    );
}

#[test]
fn wildcard_question_mark() {
    assert_eq!(
        searchb_fn(&[
            Value::Text("h?llo".to_string()),
            Value::Text("Hello".to_string())
        ]),
        Value::Number(1.0)
    );
}

#[test]
fn wildcard_star() {
    assert_eq!(
        searchb_fn(&[
            Value::Text("h*o".to_string()),
            Value::Text("Hello".to_string())
        ]),
        Value::Number(1.0)
    );
}

#[test]
fn with_start_position() {
    assert_eq!(
        searchb_fn(&[
            Value::Text("o".to_string()),
            Value::Text("Hello World".to_string()),
            Value::Number(6.0)
        ]),
        Value::Number(8.0)
    );
}
