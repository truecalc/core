use proptest::prelude::*;
use truecalc_core::{evaluate, Value};
use std::collections::HashMap;

const CASES: u32 = 500;

fn make_single_row_db(val: f64) -> Value {
    Value::Array(vec![
        Value::Array(vec![Value::Text("x".to_string())]),
        Value::Array(vec![Value::Number(val)]),
    ])
}

fn make_exact_criteria(val: f64) -> Value {
    Value::Array(vec![
        Value::Array(vec![Value::Text("x".to_string())]),
        Value::Array(vec![Value::Number(val)]),
    ])
}

fn run_db(formula: &str, db: Value, crit: Value) -> Value {
    let mut vars = HashMap::new();
    vars.insert("db".to_string(), db);
    vars.insert("crit".to_string(), crit);
    evaluate(formula, &vars)
}

fn positive_f64() -> impl Strategy<Value = f64> {
    1.0f64..=10000.0f64
}

// 1. DSUM of single-row database with exact-match criteria = that value
#[test]
fn dsum_single_row_exact_match() {
    proptest!(ProptestConfig::with_cases(CASES), |(val in positive_f64())| {
        let result = run_db("=DSUM(db, 1, crit)",
            make_single_row_db(val), make_exact_criteria(val));
        prop_assert_eq!(result, Value::Number(val));
    });
    eprintln!("proptest: {CASES} cases (val ∈ [1, 10000])");
}

// 2. DAVERAGE of single-row database = that value
#[test]
fn daverage_single_row_exact_match() {
    proptest!(ProptestConfig::with_cases(CASES), |(val in positive_f64())| {
        let result = run_db("=DAVERAGE(db, 1, crit)",
            make_single_row_db(val), make_exact_criteria(val));
        prop_assert_eq!(result, Value::Number(val));
    });
    eprintln!("proptest: {CASES} cases (val ∈ [1, 10000])");
}

// 3. DMAX of single-row database = that value
#[test]
fn dmax_single_row_exact_match() {
    proptest!(ProptestConfig::with_cases(CASES), |(val in positive_f64())| {
        let result = run_db("=DMAX(db, 1, crit)",
            make_single_row_db(val), make_exact_criteria(val));
        prop_assert_eq!(result, Value::Number(val));
    });
    eprintln!("proptest: {CASES} cases (val ∈ [1, 10000])");
}

// 4. DMIN of single-row database = that value
#[test]
fn dmin_single_row_exact_match() {
    proptest!(ProptestConfig::with_cases(CASES), |(val in positive_f64())| {
        let result = run_db("=DMIN(db, 1, crit)",
            make_single_row_db(val), make_exact_criteria(val));
        prop_assert_eq!(result, Value::Number(val));
    });
    eprintln!("proptest: {CASES} cases (val ∈ [1, 10000])");
}

// 5. DCOUNT of single-row numeric database with exact criteria = 1
#[test]
fn dcount_single_row_returns_one() {
    proptest!(ProptestConfig::with_cases(CASES), |(val in positive_f64())| {
        let result = run_db("=DCOUNT(db, 1, crit)",
            make_single_row_db(val), make_exact_criteria(val));
        prop_assert_eq!(result, Value::Number(1.0));
    });
    eprintln!("proptest: {CASES} cases (val ∈ [1, 10000])");
}

// 6. DMAX >= DMIN for a two-row database
#[test]
fn dmax_gte_dmin_two_rows() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in 1.0f64..=5000.0f64, b in 1.0f64..=5000.0f64)| {
        // Two-row database with no filtering criteria (match-all: "x", ">0")
        let db = Value::Array(vec![
            Value::Array(vec![Value::Text("x".to_string())]),
            Value::Array(vec![Value::Number(a)]),
            Value::Array(vec![Value::Number(b)]),
        ]);
        let crit = Value::Array(vec![
            Value::Array(vec![Value::Text("x".to_string())]),
            Value::Array(vec![Value::Text(">0".to_string())]),
        ]);
        let dmax = {
            let mut vars = HashMap::new();
            vars.insert("db".to_string(), db.clone());
            vars.insert("crit".to_string(), crit.clone());
            evaluate("=DMAX(db, 1, crit)", &vars)
        };
        let dmin = {
            let mut vars = HashMap::new();
            vars.insert("db".to_string(), db);
            vars.insert("crit".to_string(), crit);
            evaluate("=DMIN(db, 1, crit)", &vars)
        };
        if let (Value::Number(mx), Value::Number(mn)) = (dmax, dmin) {
            prop_assert!(mx >= mn - 1e-12, "DMAX={} < DMIN={}", mx, mn);
        }
    });
    eprintln!("proptest: {CASES} cases (a,b ∈ [1, 5000])");
}

// Sanity check
#[test]
fn sanity_dsum() {
    let db = Value::Array(vec![
        Value::Array(vec![Value::Text("val".to_string())]),
        Value::Array(vec![Value::Number(10.0)]),
        Value::Array(vec![Value::Number(20.0)]),
    ]);
    let crit = Value::Array(vec![
        Value::Array(vec![Value::Text("val".to_string())]),
        Value::Array(vec![Value::Text(">0".to_string())]),
    ]);
    let mut vars = HashMap::new();
    vars.insert("db".to_string(), db);
    vars.insert("crit".to_string(), crit);
    assert_eq!(evaluate("=DSUM(db, 1, crit)", &vars), Value::Number(30.0));
}
