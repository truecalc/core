use super::super::*;
use crate::types::Value;

#[test]
fn rank_eq_single_element() {
    // RANK.EQ(5, [5]) = 1
    assert_eq!(
        rank_eq_fn(&[Value::Number(5.0), Value::Number(5.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn rank_eq_lowest_descending() {
    // RANK.EQ(1, [1,2,3,4,5]) = 5
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_eq_fn(&[Value::Number(1.0), arr]), Value::Number(5.0));
}

#[test]
fn rank_eq_ties_ascending() {
    // RANK.EQ(3, [1,2,3,3,5], 1) = 3 (ascending, ties share rank 3)
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(3.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_eq_fn(&[Value::Number(3.0), arr, Value::Number(1.0)]), Value::Number(3.0));
}
