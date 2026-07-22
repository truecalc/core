use super::super::query_fn;
use super::{employees, num, text};
use crate::types::Value;

#[test]
fn group_by_sum() {
    let result = query_fn(&[employees(), text("select Col2, sum(Col3) group by Col2"), num(1.0)]);
    // Groups sorted ascending by Dept: Eng, HR, Sales
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Dept"), text("sum Salary")]),
            Value::Array(vec![text("Eng"), num(300.0)]),
            Value::Array(vec![text("HR"), num(300.0)]),
            Value::Array(vec![text("Sales"), num(200.0)]),
        ])
    );
}

#[test]
fn group_by_count_avg_max_min() {
    let count = query_fn(&[employees(), text("select Col2, count(Col3) group by Col2"), num(1.0)]);
    assert_eq!(
        count,
        Value::Array(vec![
            Value::Array(vec![text("Dept"), text("count Salary")]),
            Value::Array(vec![text("Eng"), num(2.0)]),
            Value::Array(vec![text("HR"), num(1.0)]),
            Value::Array(vec![text("Sales"), num(2.0)]),
        ])
    );

    let avg = query_fn(&[employees(), text("select Col2, avg(Col3) group by Col2"), num(1.0)]);
    assert_eq!(
        avg,
        Value::Array(vec![
            Value::Array(vec![text("Dept"), text("avg Salary")]),
            Value::Array(vec![text("Eng"), num(150.0)]),
            Value::Array(vec![text("HR"), num(300.0)]),
            Value::Array(vec![text("Sales"), num(100.0)]),
        ])
    );

    let max = query_fn(&[employees(), text("select Col2, max(Col3) group by Col2"), num(1.0)]);
    assert_eq!(
        max,
        Value::Array(vec![
            Value::Array(vec![text("Dept"), text("max Salary")]),
            Value::Array(vec![text("Eng"), num(200.0)]),
            Value::Array(vec![text("HR"), num(300.0)]),
            Value::Array(vec![text("Sales"), num(150.0)]),
        ])
    );

    let min = query_fn(&[employees(), text("select Col2, min(Col3) group by Col2"), num(1.0)]);
    assert_eq!(
        min,
        Value::Array(vec![
            Value::Array(vec![text("Dept"), text("min Salary")]),
            Value::Array(vec![text("Eng"), num(100.0)]),
            Value::Array(vec![text("HR"), num(300.0)]),
            Value::Array(vec![text("Sales"), num(50.0)]),
        ])
    );
}

#[test]
fn aggregate_without_group_by_collapses_to_one_row() {
    let result = query_fn(&[employees(), text("select sum(Col3)"), num(1.0)]);
    assert_eq!(result, Value::Array(vec![Value::Array(vec![text("sum Salary")]), Value::Array(vec![num(800.0)])]));
}

#[test]
fn where_then_group_by() {
    let result = query_fn(&[employees(), text("select Col2, sum(Col3) where Col3 > 60 group by Col2"), num(1.0)]);
    // Dave (50) excluded by WHERE before grouping.
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Array(vec![text("Dept"), text("sum Salary")]),
            Value::Array(vec![text("Eng"), num(300.0)]),
            Value::Array(vec![text("HR"), num(300.0)]),
            Value::Array(vec![text("Sales"), num(150.0)]),
        ])
    );
}
