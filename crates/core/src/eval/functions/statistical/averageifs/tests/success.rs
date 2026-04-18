use super::super::averageifs_fn;
use crate::types::Value;

#[test]
fn single_matching_row_returns_value() {
    // avg_range=42.0 (scalar), crit_range=10.0 (scalar), criteria=10.0
    // flatten(42.0)=[42.0], flatten(10.0)=[10.0], 10.0 matches 10.0 → average(42.0)=42.0
    assert_eq!(
        averageifs_fn(&[Value::Number(42.0), Value::Number(10.0), Value::Number(10.0)]),
        Value::Number(42.0)
    );
}

#[test]
fn text_criteria_with_operator() {
    // avg_range=5.0, crit_range=10.0, criteria=">5" → 10.0 > 5 → match → average(5.0)=5.0
    assert_eq!(
        averageifs_fn(&[
            Value::Number(5.0),
            Value::Number(10.0),
            Value::Text(">5".to_string())
        ]),
        Value::Number(5.0)
    );
}

#[test]
fn bool_criteria_match() {
    // avg_range=3.0, crit_range=TRUE, criteria=TRUE → match → average(3.0)=3.0
    assert_eq!(
        averageifs_fn(&[Value::Number(3.0), Value::Bool(true), Value::Bool(true)]),
        Value::Number(3.0)
    );
}
