use super::super::sumifs_fn;
use crate::types::Value;

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

fn texts(ss: &[&str]) -> Value {
    Value::Array(ss.iter().map(|s| Value::Text(s.to_string())).collect())
}

#[test]
fn single_criterion_gt() {
    // SUMIFS({1,2,3,4,5}, {1,2,3,4,5}, ">2") → 12 (3+4+5)
    let range = nums(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let result = sumifs_fn(&[
        range.clone(),
        range.clone(),
        Value::Text(">2".to_string()),
    ]);
    assert_eq!(result, Value::Number(12.0));
}

#[test]
fn two_criteria() {
    // SUMIFS({1,2,3,4,5}, {1,2,3,4,5}, ">1", {1,2,3,4,5}, "<5") → 9 (2+3+4)
    let range = nums(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let result = sumifs_fn(&[
        range.clone(),
        range.clone(),
        Value::Text(">1".to_string()),
        range.clone(),
        Value::Text("<5".to_string()),
    ]);
    assert_eq!(result, Value::Number(9.0));
}

#[test]
fn text_criterion_on_sum_range() {
    // SUMIFS({10,20,30}, {"a","b","a"}, "a") → 40 (10+30)
    let result = sumifs_fn(&[
        nums(&[10.0, 20.0, 30.0]),
        texts(&["a", "b", "a"]),
        Value::Text("a".to_string()),
    ]);
    assert_eq!(result, Value::Number(40.0));
}

#[test]
fn two_criteria_different_ranges() {
    // SUMIFS({100,200,300,400}, {1,2,3,4}, ">1", {10,20,30,40}, ">20") → 700 (300+400)
    let result = sumifs_fn(&[
        nums(&[100.0, 200.0, 300.0, 400.0]),
        nums(&[1.0, 2.0, 3.0, 4.0]),
        Value::Text(">1".to_string()),
        nums(&[10.0, 20.0, 30.0, 40.0]),
        Value::Text(">20".to_string()),
    ]);
    assert_eq!(result, Value::Number(700.0));
}

#[test]
fn exact_match_criterion() {
    // SUMIFS({10,20,30}, {1,2,3}, 2) → 20
    let result = sumifs_fn(&[
        nums(&[10.0, 20.0, 30.0]),
        nums(&[1.0, 2.0, 3.0]),
        Value::Number(2.0),
    ]);
    assert_eq!(result, Value::Number(20.0));
}

#[test]
fn wildcard_criterion() {
    // SUMIFS({10,20,30}, {"apple","apricot","banana"}, "a*") → 30 (10+20)
    let result = sumifs_fn(&[
        nums(&[10.0, 20.0, 30.0]),
        texts(&["apple", "apricot", "banana"]),
        Value::Text("a*".to_string()),
    ]);
    assert_eq!(result, Value::Number(30.0));
}
