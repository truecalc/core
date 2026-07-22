use super::super::query_fn;
use super::{employees, num, text};
use crate::types::Value;

#[test]
fn order_by_single_column_desc() {
    let result = query_fn(&[employees(), text("select Col1 order by Col3 desc"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Name")]),
            Value::Array(vec![text("Eve")]),
            Value::Array(vec![text("Bob")]),
            Value::Array(vec![text("Carol")]),
            Value::Array(vec![text("Alice")]),
            Value::Array(vec![text("Dave")]),
        ])
    );
}

#[test]
fn order_by_multi_key() {
    let result = query_fn(&[employees(), text("select Col1 order by Col2 asc, Col3 desc"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Name")]),
            Value::Array(vec![text("Bob")]),
            Value::Array(vec![text("Alice")]),
            Value::Array(vec![text("Eve")]),
            Value::Array(vec![text("Carol")]),
            Value::Array(vec![text("Dave")]),
        ])
    );
}

#[test]
fn limit_truncates_after_ordering() {
    let result = query_fn(&[employees(), text("select Col1 order by Col3 desc limit 3"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Name")]),
            Value::Array(vec![text("Eve")]),
            Value::Array(vec![text("Bob")]),
            Value::Array(vec![text("Carol")]),
        ])
    );
}

#[test]
fn group_by_order_by_aggregate() {
    let result = query_fn(&[employees(), text("select Col2, sum(Col3) group by Col2 order by sum(Col3) asc"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Dept"), text("sum Salary")]),
            Value::Array(vec![text("Sales"), num(200.0)]),
            Value::Array(vec![text("Eng"), num(300.0)]),
            Value::Array(vec![text("HR"), num(300.0)]),
        ])
    );
}
