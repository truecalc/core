use super::super::*;
use crate::types::Value;

#[test]
fn replace_with_empty_string() {
    assert_eq!(
        replaceb_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Text("".to_string())
        ]),
        Value::Text("Ho".to_string())
    );
}
