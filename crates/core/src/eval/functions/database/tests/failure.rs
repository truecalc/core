use super::super::dsum_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(dsum_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_two_args() {
    assert_eq!(
        dsum_fn(&[Value::Number(1.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn wrong_arity_four_args() {
    assert_eq!(
        dsum_fn(&[
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
        ]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn non_array_database_returns_value_error() {
    // database must be an array
    assert_eq!(
        dsum_fn(&[
            Value::Number(1.0),
            Value::Text("Sales".to_string()),
            Value::Number(1.0),
        ]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn invalid_field_name_returns_value_error() {
    let db = Value::Array(vec![
        Value::Array(vec![
            Value::Text("Name".to_string()),
            Value::Text("Age".to_string()),
        ]),
        Value::Array(vec![Value::Text("Alice".to_string()), Value::Number(30.0)]),
    ]);
    let criteria = Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Alice".to_string())]),
    ]);
    // "Bogus" is not a column in the database
    let result = dsum_fn(&[db, Value::Text("Bogus".to_string()), criteria]);
    assert_eq!(result, Value::Error(ErrorKind::Value));
}

#[test]
fn field_index_out_of_range_returns_value_error() {
    let db = Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Alice".to_string())]),
    ]);
    let criteria = Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Alice".to_string())]),
    ]);
    // field=5 is beyond the single column
    let result = dsum_fn(&[db, Value::Number(5.0), criteria]);
    assert_eq!(result, Value::Error(ErrorKind::Value));
}
