use super::super::query_fn;
use super::{employees, num, table, text};
use crate::types::Value;

#[test]
fn select_two_columns_with_header() {
    let result = query_fn(&[employees(), text("select Col1, Col3"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Name"), text("Salary")]),
            Value::Array(vec![text("Alice"), num(100.0)]),
            Value::Array(vec![text("Bob"), num(200.0)]),
            Value::Array(vec![text("Carol"), num(150.0)]),
            Value::Array(vec![text("Dave"), num(50.0)]),
            Value::Array(vec![text("Eve"), num(300.0)]),
        ])
    );
}

#[test]
fn select_is_case_insensitive_on_column_refs() {
    let result = query_fn(&[employees(), text("SELECT col1"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Name")]),
            Value::Array(vec![text("Alice")]),
            Value::Array(vec![text("Bob")]),
            Value::Array(vec![text("Carol")]),
            Value::Array(vec![text("Dave")]),
            Value::Array(vec![text("Eve")]),
        ])
    );
}

#[test]
fn default_select_all_columns_when_select_clause_omitted() {
    let result = query_fn(&[employees(), text("where Col3 > 100"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Name"), text("Dept"), text("Salary")]),
            Value::Array(vec![text("Bob"), text("Eng"), num(200.0)]),
            Value::Array(vec![text("Carol"), text("Sales"), num(150.0)]),
            Value::Array(vec![text("Eve"), text("HR"), num(300.0)]),
        ])
    );
}

#[test]
fn no_headers_no_label_omits_result_header_row() {
    let data = table(vec![vec![num(1.0), num(2.0)], vec![num(3.0), num(4.0)]]);
    let result = query_fn(&[data, text("select Col1")]);
    assert_eq!(result, Value::Array(vec![Value::Array(vec![num(1.0)]), Value::Array(vec![num(3.0)])]));
}
