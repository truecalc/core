use super::super::*;
use crate::types::Value;

#[test]
fn empty_find_text() {
    assert_eq!(
        findb_fn(&[
            Value::Text("".to_string()),
            Value::Text("Hello".to_string())
        ]),
        Value::Number(1.0)
    );
}
