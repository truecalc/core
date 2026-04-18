use super::super::averagea_fn;
use crate::types::Value;

#[test]
fn average_of_numbers() {
    assert_eq!(
        averagea_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(2.0)
    );
}

#[test]
fn average_single_number() {
    assert_eq!(averagea_fn(&[Value::Number(7.0)]), Value::Number(7.0));
}

#[test]
fn average_with_true_and_false() {
    // TRUE=1, FALSE=0 → (1 + 0 + 1) / 3 = 2/3
    assert_eq!(
        averagea_fn(&[Value::Bool(true), Value::Bool(false), Value::Number(1.0)]),
        Value::Number(2.0 / 3.0)
    );
}

#[test]
fn empty_values_skipped() {
    assert_eq!(
        averagea_fn(&[Value::Empty, Value::Number(4.0), Value::Empty]),
        Value::Number(4.0)
    );
}
