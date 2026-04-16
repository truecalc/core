use super::super::*;
use crate::types::Value;

#[test]
fn find_o_in_hello_world() {
    assert_eq!(
        findb_fn(&[
            Value::Text("o".to_string()),
            Value::Text("Hello World".to_string())
        ]),
        Value::Number(5.0)
    );
}

#[test]
fn find_world_in_hello_world() {
    assert_eq!(
        findb_fn(&[
            Value::Text("World".to_string()),
            Value::Text("Hello World".to_string())
        ]),
        Value::Number(7.0)
    );
}

#[test]
fn find_with_start_position() {
    assert_eq!(
        findb_fn(&[
            Value::Text("o".to_string()),
            Value::Text("Hello World".to_string()),
            Value::Number(6.0)
        ]),
        Value::Number(8.0)
    );
}
