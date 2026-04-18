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
fn sum_without_sum_range() {
    // SUMIF(range, ">2") → 3+4+5 = 12
    let vars: HashMap<_, _> = [nums_var("R", &[1.0, 2.0, 3.0, 4.0, 5.0])].into();
    assert_eq!(run("=SUMIF(R,\">2\")", vars), Value::Number(12.0));
}

#[test]
fn sum_with_sum_range() {
    // SUMIF(range, "a", sum_range) → 10+30 = 40
    let vars: HashMap<_, _> = [
        texts_var("R", &["a", "b", "a", "c"]),
        nums_var("S", &[10.0, 20.0, 30.0, 40.0]),
    ].into();
    assert_eq!(run("=SUMIF(R,\"a\",S)", vars), Value::Number(40.0));
}

#[test]
fn sum_exact_number_criterion() {
    // SUMIF(range, 2, sum_range) → 20+20 = 40
    let vars: HashMap<_, _> = [
        nums_var("R", &[1.0, 2.0, 3.0, 2.0, 1.0]),
        nums_var("S", &[10.0, 20.0, 30.0, 20.0, 10.0]),
    ].into();
    assert_eq!(run("=SUMIF(R,2,S)", vars), Value::Number(40.0));
}

#[test]
fn sum_gt_criterion() {
    // SUMIF(range, ">3") → 4+5 = 9
    let vars: HashMap<_, _> = [nums_var("R", &[1.0, 2.0, 3.0, 4.0, 5.0])].into();
    assert_eq!(run("=SUMIF(R,\">3\")", vars), Value::Number(9.0));
}

#[test]
fn sum_lte_criterion() {
    // SUMIF(range, "<=2") → 1+2 = 3
    let vars: HashMap<_, _> = [nums_var("R", &[1.0, 2.0, 3.0, 4.0, 5.0])].into();
    assert_eq!(run("=SUMIF(R,\"<=2\")", vars), Value::Number(3.0));
}

#[test]
fn sum_ne_criterion_with_sum_range() {
    // SUMIF(range, "<>2", sum_range) → 10+30 = 40
    let vars: HashMap<_, _> = [
        nums_var("R", &[1.0, 2.0, 3.0]),
        nums_var("S", &[10.0, 20.0, 30.0]),
    ].into();
    assert_eq!(run("=SUMIF(R,\"<>2\",S)", vars), Value::Number(40.0));
}

#[test]
fn sum_wildcard_criterion() {
    // SUMIF(range, "ap*", sum_range) → 10+30 = 40
    let vars: HashMap<_, _> = [
        texts_var("R", &["apple", "banana", "apricot"]),
        nums_var("S", &[10.0, 20.0, 30.0]),
    ].into();
    assert_eq!(run("=SUMIF(R,\"ap*\",S)", vars), Value::Number(40.0));
}

#[test]
fn array_literal_with_sum_range_returns_na() {
    // Inline array constants with sum_range → #N/A (Google Sheets behavior)
    let vars = HashMap::new();
    assert_eq!(
        run("=SUMIF({1,2,3},\">2\",{10,20,30})", vars),
        Value::Error(crate::types::ErrorKind::NA)
    );
}

#[test]
fn array_literal_without_sum_range_works() {
    // Inline array constants WITHOUT sum_range → works in Google Sheets
    let vars = HashMap::new();
    assert_eq!(run("=SUMIF({1,2,3,4,5},\">2\")", vars), Value::Number(12.0));
}
