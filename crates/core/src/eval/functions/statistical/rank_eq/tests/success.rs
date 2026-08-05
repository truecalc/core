use super::super::*;
use crate::types::Value;

#[test]
fn rank_eq_descending_default() {
    // RANK.EQ(3, [1,2,3,4,5]) descending: values > 3 = {4,5} → rank 3
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_eq_fn(&[Value::Number(3.0), arr]), Value::Number(3.0));
}

#[test]
fn rank_eq_ties_get_lowest_rank() {
    // RANK.EQ(3, [1,2,3,3,5]) descending: values > 3 = {5} → rank 2 (ties share rank 2)
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(3.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_eq_fn(&[Value::Number(3.0), arr]), Value::Number(2.0));
}

#[test]
fn rank_eq_ascending() {
    // RANK.EQ(3, [1,2,3,4,5], 1) ascending: values < 3 = {1,2} → rank 3
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_eq_fn(&[Value::Number(3.0), arr, Value::Number(1.0)]), Value::Number(3.0));
}

#[test]
fn rank_eq_highest_descending() {
    // RANK.EQ(5, [1,2,3,4,5]) = 1
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_eq_fn(&[Value::Number(5.0), arr]), Value::Number(1.0));
}
