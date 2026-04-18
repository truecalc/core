use super::super::*;
use crate::types::Value;

#[test]
fn rank_avg_no_tie() {
    // RANK.AVG(3, [1,2,3,4,5]) = 3.0 (no tie, same as RANK.EQ)
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_avg_fn(&[Value::Number(3.0), arr]), Value::Number(3.0));
}

#[test]
fn rank_avg_with_tie_descending() {
    // RANK.AVG(3, [1,2,3,3,5]) descending:
    // count_better = count(>3) = 1 (just 5), count_equal = 2
    // avg_rank = (1+1) + (2-1)/2 = 2 + 0.5 = 2.5
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(3.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_avg_fn(&[Value::Number(3.0), arr]), Value::Number(2.5));
}

#[test]
fn rank_avg_ascending() {
    // RANK.AVG(3, [1,2,3,4,5], 1) = 3.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_avg_fn(&[Value::Number(3.0), arr, Value::Number(1.0)]), Value::Number(3.0));
}

#[test]
fn rank_avg_tie_ascending() {
    // RANK.AVG(3, [1,2,3,3,5], 1) ascending:
    // count_better = count(<3) = 2 (1,2), count_equal = 2
    // avg_rank = (2+1) + (2-1)/2 = 3 + 0.5 = 3.5
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(3.0), Value::Number(5.0),
    ]);
    assert_eq!(rank_avg_fn(&[Value::Number(3.0), arr, Value::Number(1.0)]), Value::Number(3.5));
}
