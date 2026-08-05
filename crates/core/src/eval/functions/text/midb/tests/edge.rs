use super::super::*;
use crate::types::Value;

#[test]
fn length_exceeds_remaining() {
    assert_eq!(
        midb_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(3.0),
            Value::Number(100.0)
        ]),
        Value::Text("llo".to_string())
    );
}

#[test]
fn start_beyond_end() {
    assert_eq!(
        midb_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(10.0),
            Value::Number(3.0)
        ]),
        Value::Text("".to_string())
    );
}
