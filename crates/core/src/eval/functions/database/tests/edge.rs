use super::super::{daverage_fn, dcount_fn, dmax_fn, dmin_fn, dproduct_fn, dstdev_fn, dsum_fn};
use crate::types::{ErrorKind, Value};

fn simple_db() -> Value {
    Value::Array(vec![
        Value::Array(vec![
            Value::Text("Name".to_string()),
            Value::Text("Sales".to_string()),
        ]),
        Value::Array(vec![Value::Text("Alice".to_string()), Value::Number(100.0)]),
    ])
}

fn criteria_all() -> Value {
    Value::Array(vec![
        Value::Array(vec![Value::Text("Sales".to_string())]),
        Value::Array(vec![Value::Text(">=0".to_string())]),
    ])
}

#[test]
fn no_matching_rows_dsum_returns_zero() {
    let db = simple_db();
    let criteria_no_match = Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Charlie".to_string())]),
    ]);
    let result = dsum_fn(&[db, Value::Text("Sales".to_string()), criteria_no_match]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn no_matching_rows_dmax_returns_zero() {
    let db = simple_db();
    let criteria_no_match = Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Charlie".to_string())]),
    ]);
    let result = dmax_fn(&[db, Value::Text("Sales".to_string()), criteria_no_match]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn no_matching_rows_dmin_returns_zero() {
    let db = simple_db();
    let criteria_no_match = Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Charlie".to_string())]),
    ]);
    let result = dmin_fn(&[db, Value::Text("Sales".to_string()), criteria_no_match]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn no_matching_rows_dproduct_returns_zero() {
    let db = simple_db();
    let criteria_no_match = Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Charlie".to_string())]),
    ]);
    let result = dproduct_fn(&[db, Value::Text("Sales".to_string()), criteria_no_match]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn no_matching_rows_daverage_returns_div_by_zero() {
    let db = simple_db();
    let criteria_no_match = Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Charlie".to_string())]),
    ]);
    let result = daverage_fn(&[db, Value::Text("Sales".to_string()), criteria_no_match]);
    assert_eq!(result, Value::Error(ErrorKind::DivByZero));
}

#[test]
fn dstdev_single_match_returns_div_by_zero() {
    // sample stdev requires at least 2 values
    let result = dstdev_fn(&[simple_db(), Value::Text("Sales".to_string()), criteria_all()]);
    assert_eq!(result, Value::Error(ErrorKind::DivByZero));
}

#[test]
fn dcount_counts_only_numeric_values() {
    // Name column contains text — DCOUNT of Name should return 0
    let result = dcount_fn(&[simple_db(), Value::Text("Name".to_string()), criteria_all()]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn field_lookup_is_case_insensitive() {
    // "sales" (lowercase) should resolve to "Sales" column
    let result = dsum_fn(&[simple_db(), Value::Text("sales".to_string()), criteria_all()]);
    assert_eq!(result, Value::Number(100.0));
}

#[test]
fn database_with_only_header_row_returns_zero() {
    // No data rows → sum = 0
    let db = Value::Array(vec![Value::Array(vec![
        Value::Text("Name".to_string()),
        Value::Text("Sales".to_string()),
    ])]);
    let criteria = Value::Array(vec![
        Value::Array(vec![Value::Text("Name".to_string())]),
        Value::Array(vec![Value::Text("Alice".to_string())]),
    ]);
    let result = dsum_fn(&[db, Value::Text("Sales".to_string()), criteria]);
    assert_eq!(result, Value::Number(0.0));
}
