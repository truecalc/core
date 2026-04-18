use super::super::*;
use crate::types::Value;

#[test]
fn rank_avg_single_element() {
    // RANK.AVG(5, [5]) = 1.0
    assert_eq!(
        rank_avg_fn(&[Value::Number(5.0), Value::Number(5.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn rank_avg_triple_tie_descending() {
    // RANK.AVG(3, [3,3,3]) descending: all tie at ranks 1,2,3 → avg 2.0
    let arr = Value::Array(vec![
        Value::Number(3.0), Value::Number(3.0), Value::Number(3.0),
    ]);
    assert_eq!(rank_avg_fn(&[Value::Number(3.0), arr]), Value::Number(2.0));
}

#[test]
fn rank_avg_highest_value_desc() {
    // RANK.AVG(5, [1,2,3,4,5]) = 1.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_avg_fn(&[Value::Number(5.0), arr]), Value::Number(1.0));
}
