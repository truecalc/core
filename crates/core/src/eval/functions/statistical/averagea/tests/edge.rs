use super::super::averagea_fn;
use crate::types::Value;

#[test]
fn mixed_numbers_and_booleans() {
    // TRUE=1, FALSE=0, 5 → (1 + 0 + 5) / 3 = 2.0
    assert_eq!(
        averagea_fn(&[Value::Bool(true), Value::Bool(false), Value::Number(5.0)]),
        Value::Number(2.0)
    );
}

#[test]
fn only_booleans() {
    // TRUE=1, TRUE=1 → 2/2 = 1.0
    assert_eq!(
        averagea_fn(&[Value::Bool(true), Value::Bool(true)]),
        Value::Number(1.0)
    );
}

#[test]
fn all_false_booleans() {
    // FALSE=0, FALSE=0 → 0/2 = 0.0
    assert_eq!(
        averagea_fn(&[Value::Bool(false), Value::Bool(false)]),
        Value::Number(0.0)
    );
}
