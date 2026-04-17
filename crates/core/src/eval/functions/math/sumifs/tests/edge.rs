use crate::types::Value;
use std::collections::HashMap;

fn run(formula: &str, vars: HashMap<String, Value>) -> Value {
    crate::evaluate(formula, &vars)
}

fn nums_var(name: &str, ns: &[f64]) -> (String, Value) {
    (name.to_string(), Value::Array(ns.iter().map(|&n| Value::Number(n)).collect()))
}

fn texts_var(name: &str, ss: &[&str]) -> (String, Value) {
    (name.to_string(), Value::Array(ss.iter().map(|s| Value::Text(s.to_string())).collect()))
}

#[test]
fn no_matches_returns_zero() {
    // SUMIFS(range, range, ">10") → 0
    let vars: HashMap<_, _> = [nums_var("R", &[1.0, 2.0, 3.0])].into();
    assert_eq!(run("=SUMIFS(R,R,\">10\")", vars), Value::Number(0.0));
}

#[test]
fn ne_criterion() {
    // SUMIFS(sum, range, "<>2") → 40 (10+30)
    let vars: HashMap<_, _> = [
        nums_var("S", &[10.0, 20.0, 30.0]),
        nums_var("R", &[1.0, 2.0, 3.0]),
    ].into();
    assert_eq!(run("=SUMIFS(S,R,\"<>2\")", vars), Value::Number(40.0));
}

#[test]
fn empty_sum_range_returns_zero() {
    let vars: HashMap<_, _> = [
        ("S".to_string(), Value::Array(vec![])),
        ("R".to_string(), Value::Array(vec![])),
    ].into();
    assert_eq!(run("=SUMIFS(S,R,1)", vars), Value::Number(0.0));
}

#[test]
fn two_criteria_text_and_number() {
    // SUMIFS(sum, r1, "a", r2, 1) → 400 (100+300)
    let vars: HashMap<_, _> = [
        nums_var("S", &[100.0, 200.0, 300.0, 400.0]),
        texts_var("R1", &["a", "b", "a", "b"]),
        nums_var("R2", &[1.0, 2.0, 1.0, 2.0]),
    ].into();
    assert_eq!(run("=SUMIFS(S,R1,\"a\",R2,1)", vars), Value::Number(400.0));
}
