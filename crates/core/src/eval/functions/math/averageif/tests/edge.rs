use super::super::averageif_fn;
use crate::types::{ErrorKind, Value};

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

// When the range is an array literal and a separate avg_range is provided,
// Google Sheets returns #N/A.
#[test]
fn three_arg_with_array_range_returns_na() {
    let result = averageif_fn(&[
        nums(&[1.0, 2.0, 3.0, 4.0]),
        Value::Text(">0".to_string()),
        nums(&[10.0, 20.0]),
    ]);
    assert_eq!(result, Value::Error(ErrorKind::NA));
}

#[test]
fn scalar_range_matched() {
    // AVERAGEIF(5, ">=5", 100) — scalar range (not array) with avg_range: OK → 100
    let result = averageif_fn(&[
        Value::Number(5.0),
        Value::Text(">=5".to_string()),
        Value::Number(100.0),
    ]);
    assert_eq!(result, Value::Number(100.0));
}

#[test]
fn single_match_is_value_itself() {
    // Average of a single matched value in 2-arg form is that value.
    let result = averageif_fn(&[nums(&[1.0, 7.0, 3.0]), Value::Text("=7".to_string())]);
    assert_eq!(result, Value::Number(7.0));
}
