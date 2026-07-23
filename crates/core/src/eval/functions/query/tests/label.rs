use super::super::query_fn;
use super::{employees, num, table, text};
use crate::types::Value;

#[test]
fn label_overrides_default_header() {
    let result = query_fn(&[employees(), text("select Col1, Col3 label Col3 'Pay'"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Name"), text("Pay")]),
            Value::Array(vec![text("Alice"), num(100.0)]),
            Value::Array(vec![text("Bob"), num(200.0)]),
            Value::Array(vec![text("Carol"), num(150.0)]),
            Value::Array(vec![text("Dave"), num(50.0)]),
            Value::Array(vec![text("Eve"), num(300.0)]),
        ])
    );
}

#[test]
fn label_on_aggregate() {
    let result = query_fn(&[employees(), text("select Col2, sum(Col3) group by Col2 label sum(Col3) 'Total Pay'"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Dept"), text("Total Pay")]),
            Value::Array(vec![text("Eng"), num(300.0)]),
            Value::Array(vec![text("HR"), num(300.0)]),
            Value::Array(vec![text("Sales"), num(200.0)]),
        ])
    );
}

#[test]
fn label_forces_header_row_even_without_headers_arg() {
    let data = table(vec![vec![num(1.0), num(2.0)], vec![num(3.0), num(4.0)]]);
    let result = query_fn(&[data, text("select Col1 label Col1 'X'")]);
    assert_eq!(result, Value::Array(vec![Value::Array(vec![text("X")]), Value::Array(vec![num(1.0)]), Value::Array(vec![num(3.0)])]));
}
