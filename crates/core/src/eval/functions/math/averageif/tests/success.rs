use super::super::averageif_fn;
use crate::types::{ErrorKind, Value};

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

#[test]
fn average_without_avg_range() {
    // AVERAGEIF({1,2,3,4,5}, ">2") — 2-arg form: array range OK → avg(3,4,5) = 4
    let result = averageif_fn(&[nums(&[1.0, 2.0, 3.0, 4.0, 5.0]), Value::Text(">2".to_string())]);
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn average_gte_criterion() {
    // AVERAGEIF({1,2,3,4,5}, ">=4") — 2-arg form → avg(4,5) = 4.5
    let result = averageif_fn(&[nums(&[1.0, 2.0, 3.0, 4.0, 5.0]), Value::Text(">=4".to_string())]);
    assert_eq!(result, Value::Number(4.5));
}

#[test]
fn average_lte_criterion() {
    // AVERAGEIF({1,2,3,4,5}, "<=2") — 2-arg form → avg(1,2) = 1.5
    let result = averageif_fn(&[nums(&[1.0, 2.0, 3.0, 4.0, 5.0]), Value::Text("<=2".to_string())]);
    assert_eq!(result, Value::Number(1.5));
}

// 3-arg form with an array as the range argument returns #N/A (Google Sheets behaviour).
#[test]
fn average_with_array_range_and_avg_range_returns_na() {
    let result = averageif_fn(&[
        nums(&[1.0, 2.0, 3.0]),
        Value::Text("a".to_string()),
        nums(&[10.0, 20.0, 30.0]),
    ]);
    assert_eq!(result, Value::Error(ErrorKind::NA));
}
