use super::super::*;
use crate::types::Value;

#[test]
fn rank_single_element_desc() {
    // RANK(5, [5]) = 1
    assert_eq!(
        rank_fn(&[Value::Number(5.0), Value::Number(5.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn rank_single_element_asc() {
    // RANK(5, [5], 1) = 1
    assert_eq!(
        rank_fn(&[Value::Number(5.0), Value::Number(5.0), Value::Number(1.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn rank_lowest_value_desc() {
    // RANK(1, [1,2,3,4,5]) = 5
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_fn(&[Value::Number(1.0), arr]), Value::Number(5.0));
}

#[test]
fn rank_highest_value_asc() {
    // RANK(5, [1,2,3,4,5], 1) = 5
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_fn(&[Value::Number(5.0), arr, Value::Number(1.0)]), Value::Number(5.0));
}
