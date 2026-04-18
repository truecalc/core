use super::super::minifs_fn;
use crate::types::Value;

#[test]
fn no_matches_returns_zero() {
    // min_range=5.0, crit_range=1.0, criteria=99.0 → no rows match → 0.0
    assert_eq!(
        minifs_fn(&[Value::Number(5.0), Value::Number(1.0), Value::Number(99.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn multiple_criteria_all_must_match() {
    // min_range=7.0, crit_range1=10.0 criteria1=10.0, crit_range2=20.0 criteria2=20.0
    // Both match → min(7.0) = 7.0
    assert_eq!(
        minifs_fn(&[
            Value::Number(7.0),
            Value::Number(10.0),
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(20.0),
        ]),
        Value::Number(7.0)
    );
}

#[test]
fn multiple_criteria_one_fails_returns_zero() {
    // min_range=7.0, crit_range1=10.0 criteria1=10.0, crit_range2=20.0 criteria2=99.0
    // Second criterion fails → no match → 0.0
    assert_eq!(
        minifs_fn(&[
            Value::Number(7.0),
            Value::Number(10.0),
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(99.0),
        ]),
        Value::Number(0.0)
    );
}
