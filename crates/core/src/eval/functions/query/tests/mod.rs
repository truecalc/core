mod select;
mod where_clause;
mod group_by;
mod order_by_limit;
mod label;
mod errors;

use crate::types::Value;

pub(crate) fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}

pub(crate) fn num(n: f64) -> Value {
    Value::Number(n)
}

pub(crate) fn table(rows: Vec<Vec<Value>>) -> Value {
    Value::Array(rows.into_iter().map(Value::Array).collect())
}

/// A small employee table: header row + 5 data rows.
/// Columns: Name (Col1), Dept (Col2), Salary (Col3).
pub(crate) fn employees() -> Value {
    table(vec![
        vec![text("Name"), text("Dept"), text("Salary")],
        vec![text("Alice"), text("Eng"), num(100.0)],
        vec![text("Bob"), text("Eng"), num(200.0)],
        vec![text("Carol"), text("Sales"), num(150.0)],
        vec![text("Dave"), text("Sales"), num(50.0)],
        vec![text("Eve"), text("HR"), num(300.0)],
    ])
}
