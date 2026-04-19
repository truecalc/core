use proptest::prelude::*;
use truecalc_core::{evaluate, Value};
use std::collections::HashMap;

const CASES: u32 = 500;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

fn small_f64() -> impl Strategy<Value = f64> {
    -1e6f64..1e6f64
}

fn col_vec(vals: &[f64]) -> Value {
    Value::Array(vals.iter().map(|&v| Value::Array(vec![Value::Number(v)])).collect())
}

fn run_sort(arr: Value, ascending: bool) -> Value {
    let mut vars = HashMap::new();
    vars.insert("arr".to_string(), arr);
    let asc = if ascending { "TRUE" } else { "FALSE" };
    evaluate(&format!("=SORT(arr, 1, {asc})"), &vars)
}

fn index_2d(arr: Value, row: usize) -> Value {
    let mut vars = HashMap::new();
    vars.insert("arr".to_string(), arr);
    evaluate(&format!("=INDEX(arr, {row}, 1)"), &vars)
}

// 1. SORT ascending: first element <= second
#[test]
fn sort_two_ascending_order() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let sorted = run_sort(col_vec(&[a, b]), true);
        let v1 = index_2d(sorted.clone(), 1);
        let v2 = index_2d(sorted, 2);
        if let (Value::Number(lo), Value::Number(hi)) = (v1, v2) {
            prop_assert!(lo <= hi + 1e-12, "SORT asc: first={} > second={}", lo, hi);
        }
    });
    eprintln!("proptest: {CASES} cases (a,b ∈ [-1e6, 1e6])");
}

// 2. SORT preserves element count
#[test]
fn sort_preserves_length() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in -100.0f64..100.0f64, b in -100.0f64..100.0f64, c in -100.0f64..100.0f64)| {
        let arr = col_vec(&[a, b, c]);
        let mut vars = HashMap::new();
        vars.insert("arr".to_string(), arr);
        let result = evaluate("=ROWS(SORT(arr))", &vars);
        prop_assert_eq!(result, Value::Number(3.0));
    });
    eprintln!("proptest: {CASES} cases (a,b,c ∈ [-100, 100])");
}

// 3. SORT descending: first element >= second
#[test]
fn sort_two_descending_order() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let sorted = run_sort(col_vec(&[a, b]), false);
        let v1 = index_2d(sorted.clone(), 1);
        let v2 = index_2d(sorted, 2);
        if let (Value::Number(hi), Value::Number(lo)) = (v1, v2) {
            prop_assert!(hi >= lo - 1e-12, "SORT desc: first={} < second={}", hi, lo);
        }
    });
    eprintln!("proptest: {CASES} cases (a,b ∈ [-1e6, 1e6])");
}

// 4. UNIQUE of repeated value returns exactly 1 row
#[test]
fn unique_of_duplicates_returns_one() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in -100.0f64..100.0f64)| {
        let arr = col_vec(&[x, x, x]);
        let mut vars = HashMap::new();
        vars.insert("arr".to_string(), arr);
        let result = evaluate("=ROWS(UNIQUE(arr))", &vars);
        prop_assert_eq!(result, Value::Number(1.0));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-100, 100])");
}

// 5. UNIQUE of two distinct values returns 2 rows
#[test]
fn unique_of_distinct_values_preserves_count() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in 1i32..=500i32, offset in 1i32..=500i32)| {
        let b = a + offset;
        let arr = col_vec(&[a as f64, b as f64]);
        let mut vars = HashMap::new();
        vars.insert("arr".to_string(), arr);
        let result = evaluate("=ROWS(UNIQUE(arr))", &vars);
        prop_assert_eq!(result, Value::Number(2.0));
    });
    eprintln!("proptest: {CASES} cases (distinct integer pairs)");
}

// 6. ROWS of a column vector matches element count
#[test]
fn rows_counts_correctly() {
    proptest!(ProptestConfig::with_cases(CASES), |(n in 1usize..=10usize)| {
        let arr = col_vec(&vec![1.0; n]);
        let mut vars = HashMap::new();
        vars.insert("arr".to_string(), arr);
        let result = evaluate("=ROWS(arr)", &vars);
        prop_assert_eq!(result, Value::Number(n as f64));
    });
    eprintln!("proptest: {CASES} cases (n ∈ [1, 10])");
}

// Sanity checks
#[test]
fn sanity_sort_ascending() {
    assert_eq!(run("=INDEX(SORT({3;1;2}), 1, 1)"), Value::Number(1.0));
    assert_eq!(run("=INDEX(SORT({3;1;2}), 3, 1)"), Value::Number(3.0));
}

#[test]
fn sanity_unique() {
    assert_eq!(run("=ROWS(UNIQUE({1;1;2}))"), Value::Number(2.0));
}

#[test]
fn sanity_rows() {
    assert_eq!(run("=ROWS({3;1;2})"), Value::Number(3.0));
}
