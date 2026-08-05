use super::super::hyperlink_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(hyperlink_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn three_args_returns_na() {
    assert_eq!(
        hyperlink_fn(&[
            Value::Text("url".to_string()),
            Value::Text("label".to_string()),
            Value::Text("extra".to_string()),
        ]),
        Value::Error(ErrorKind::NA)
    );
}
