use super::super::hyperlink_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn empty_url_returns_empty_text() {
    assert_eq!(
        hyperlink_fn(&[Value::Text(String::new())]),
        Value::Text(String::new())
    );
}

#[test]
fn wrong_arg_count_zero_returns_error() {
    assert_eq!(hyperlink_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arg_count_three_returns_error() {
    assert_eq!(
        hyperlink_fn(&[
            Value::Text("https://a.com".to_string()),
            Value::Text("label".to_string()),
            Value::Text("extra".to_string()),
        ]),
        Value::Error(ErrorKind::NA)
    );
}
