use super::super::query_fn;
use super::{employees, num, table, text};
use crate::types::Value;

#[test]
fn where_equality_on_text() {
    let result = query_fn(&[employees(), text("select Col1 where Col2 = 'Eng'"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![Value::Array(vec![text("Name")]), Value::Array(vec![text("Alice")]), Value::Array(vec![text("Bob")])])
    );
}

#[test]
fn where_greater_than_numeric() {
    let result = query_fn(&[employees(), text("select Col1 where Col3 > 150"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![Value::Array(vec![text("Name")]), Value::Array(vec![text("Bob")]), Value::Array(vec![text("Eve")])])
    );
}

#[test]
fn where_not_equal_both_spellings() {
    let ne1 = query_fn(&[employees(), text("select Col1 where Col2 != 'Eng'"), num(1.0)]);
    let ne2 = query_fn(&[employees(), text("select Col1 where Col2 <> 'Eng'"), num(1.0)]);
    assert_eq!(ne1, ne2);
    assert_eq!(
        ne1,
        Value::Array(vec![
            Value::Array(vec![text("Name")]),
            Value::Array(vec![text("Carol")]),
            Value::Array(vec![text("Dave")]),
            Value::Array(vec![text("Eve")]),
        ])
    );
}

#[test]
fn where_and_combines_conditions() {
    let result = query_fn(&[employees(), text("select Col1 where Col2 = 'Eng' and Col3 > 150"), num(1.0)]);
    assert_eq!(result, Value::Array(vec![Value::Array(vec![text("Name")]), Value::Array(vec![text("Bob")])]));
}

#[test]
fn where_or_combines_conditions() {
    let result = query_fn(&[employees(), text("select Col1 where Col2 = 'HR' or Col2 = 'Sales'"), num(1.0)]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Name")]),
            Value::Array(vec![text("Carol")]),
            Value::Array(vec![text("Dave")]),
            Value::Array(vec![text("Eve")]),
        ])
    );
}

#[test]
fn where_is_null_and_is_not_null() {
    let data = table(vec![
        vec![text("Name"), text("Score")],
        vec![text("Alice"), num(10.0)],
        vec![text("Bob"), Value::Empty],
        vec![text("Carol"), num(20.0)],
    ]);

    let is_null = query_fn(&[data.clone(), text("select Col1 where Col2 is null"), num(1.0)]);
    assert_eq!(is_null, Value::Array(vec![Value::Array(vec![text("Name")]), Value::Array(vec![text("Bob")])]));

    let is_not_null = query_fn(&[data, text("select Col1 where Col2 is not null"), num(1.0)]);
    assert_eq!(
        is_not_null,
        Value::Array(vec![Value::Array(vec![text("Name")]), Value::Array(vec![text("Alice")]), Value::Array(vec![text("Carol")])])
    );
}
