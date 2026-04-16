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
fn single_criterion_gt() {
    // SUMIFS(range, range, ">2") → 12 (3+4+5)
    let vars: HashMap<_, _> = [nums_var("R", &[1.0, 2.0, 3.0, 4.0, 5.0])].into();
    assert_eq!(run("=SUMIFS(R,R,\">2\")", vars), Value::Number(12.0));
}

#[test]
fn two_criteria() {
    // SUMIFS(range, range, ">1", range, "<5") → 9 (2+3+4)
    let vars: HashMap<_, _> = [nums_var("R", &[1.0, 2.0, 3.0, 4.0, 5.0])].into();
    assert_eq!(run("=SUMIFS(R,R,\">1\",R,\"<5\")", vars), Value::Number(9.0));
}

#[test]
fn text_criterion_on_sum_range() {
    // SUMIFS(sum, range, "a") → 40 (10+30)
    let vars: HashMap<_, _> = [
        nums_var("S", &[10.0, 20.0, 30.0]),
        texts_var("R", &["a", "b", "a"]),
    ].into();
    assert_eq!(run("=SUMIFS(S,R,\"a\")", vars), Value::Number(40.0));
}

#[test]
fn two_criteria_different_ranges() {
    // SUMIFS(sum, r1, ">1", r2, ">20") → 700 (300+400)
    let vars: HashMap<_, _> = [
        nums_var("S", &[100.0, 200.0, 300.0, 400.0]),
        nums_var("R1", &[1.0, 2.0, 3.0, 4.0]),
        nums_var("R2", &[10.0, 20.0, 30.0, 40.0]),
    ].into();
    assert_eq!(run("=SUMIFS(S,R1,\">1\",R2,\">20\")", vars), Value::Number(700.0));
}

#[test]
fn exact_match_criterion() {
    // SUMIFS(sum, range, 2) → 20
    let vars: HashMap<_, _> = [
        nums_var("S", &[10.0, 20.0, 30.0]),
        nums_var("R", &[1.0, 2.0, 3.0]),
    ].into();
    assert_eq!(run("=SUMIFS(S,R,2)", vars), Value::Number(20.0));
}

#[test]
fn wildcard_criterion() {
    // SUMIFS(sum, range, "a*") → 30 (10+20)
    let vars: HashMap<_, _> = [
        nums_var("S", &[10.0, 20.0, 30.0]),
        texts_var("R", &["apple", "apricot", "banana"]),
    ].into();
    assert_eq!(run("=SUMIFS(S,R,\"a*\")", vars), Value::Number(30.0));
}

#[test]
fn array_literal_returns_na() {
    // Inline array constants → #N/A (Google Sheets behavior)
    let vars = HashMap::new();
    assert_eq!(
        run("=SUMIFS({1,2,3},{1,2,3},\">2\")", vars),
        Value::Error(crate::types::ErrorKind::NA)
    );
}
