use super::super::maxifs_fn;
use crate::types::Value;

#[test]
fn single_matching_row_returns_value() {
    // max_range=42.0, crit_range=10.0, criteria=10.0 → match → max(42.0) = 42.0
    assert_eq!(
        maxifs_fn(&[Value::Number(42.0), Value::Number(10.0), Value::Number(10.0)]),
        Value::Number(42.0)
    );
}

#[test]
fn text_criteria_with_operator() {
    // max_range=5.0, crit_range=10.0, criteria=">5" → 10.0 > 5 → match → max(5.0)=5.0
    assert_eq!(
        maxifs_fn(&[
            Value::Number(5.0),
            Value::Number(10.0),
            Value::Text(">5".to_string())
        ]),
        Value::Number(5.0)
    );
}

#[test]
fn bool_criteria_match() {
    // max_range=3.0, crit_range=TRUE, criteria=TRUE → match → max(3.0)=3.0
    assert_eq!(
        maxifs_fn(&[Value::Number(3.0), Value::Bool(true), Value::Bool(true)]),
        Value::Number(3.0)
    );
}
