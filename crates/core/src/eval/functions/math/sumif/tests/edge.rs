use crate::types::Value;
use std::collections::HashMap;

fn run(formula: &str, vars: HashMap<String, Value>) -> Value {
    crate::evaluate(formula, &vars)
}

fn nums_var(name: &str, ns: &[f64]) -> (String, Value) {
    (name.to_string(), Value::Array(ns.iter().map(|&n| Value::Number(n)).collect()))
}

#[test]
fn no_matches_returns_zero() {
    // SUMIF(range, ">10") → 0
    let vars: HashMap<_, _> = [nums_var("R", &[1.0, 2.0, 3.0])].into();
    assert_eq!(run("=SUMIF(R,\">10\")", vars), Value::Number(0.0));
}

#[test]
fn empty_array_returns_zero() {
    let vars: HashMap<_, _> = [("R".to_string(), Value::Array(vec![]))].into();
    assert_eq!(run("=SUMIF(R,1)", vars), Value::Number(0.0));
}

#[test]
fn sum_range_shorter_than_range_uses_zip() {
    // range has 4 elements, sum_range has 2; only first 2 pairs are considered.
    let vars: HashMap<_, _> = [
        nums_var("R", &[1.0, 2.0, 3.0, 4.0]),
        nums_var("S", &[10.0, 20.0]),
    ].into();
    assert_eq!(run("=SUMIF(R,\">0\",S)", vars), Value::Number(30.0));
}

#[test]
fn numeric_text_in_sum_range_is_summed() {
    let vars: HashMap<_, _> = [
        ("R".to_string(), Value::Array(vec![Value::Number(1.0)])),
        ("S".to_string(), Value::Array(vec![Value::Text("3".to_string())])),
    ].into();
    assert_eq!(run("=SUMIF(R,\"=1\",S)", vars), Value::Number(3.0));
}

#[test]
fn scalar_range_matched() {
    // SUMIF with scalar range variable
    let vars: HashMap<_, _> = [
        ("R".to_string(), Value::Number(5.0)),
        ("S".to_string(), Value::Number(100.0)),
    ].into();
    assert_eq!(run("=SUMIF(R,\">=5\",S)", vars), Value::Number(100.0));
}
