use super::super::*;
use crate::types::Value;

#[test]
fn basic_midb() {
    assert_eq!(
        midb_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(2.0),
            Value::Number(3.0)
        ]),
        Value::Text("ell".to_string())
    );
}

#[test]
fn world_from_hello_world() {
    assert_eq!(
        midb_fn(&[
            Value::Text("Hello World".to_string()),
            Value::Number(7.0),
            Value::Number(5.0)
        ]),
        Value::Text("World".to_string())
    );
}

#[test]
fn first_char() {
    assert_eq!(
        midb_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(5.0),
            Value::Number(1.0)
        ]),
        Value::Text("o".to_string())
    );
}

#[test]
fn full_string() {
    assert_eq!(
        midb_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(1.0),
            Value::Number(5.0)
        ]),
        Value::Text("Hello".to_string())
    );
}
