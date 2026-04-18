use proptest::prelude::*;
use truecalc_core::{evaluate, Value};
use std::collections::HashMap;

const CASES: u32 = 500;

fn run_text_vars(formula: &str, vars: Vec<(&str, &str)>) -> Value {
    let map: HashMap<String, Value> = vars
        .into_iter()
        .map(|(k, v)| (k.to_string(), Value::Text(v.to_string())))
        .collect();
    evaluate(formula, &map)
}

fn ascii_string() -> impl Strategy<Value = String> {
    "[a-z]{0,20}".prop_map(|s| s)
}

fn spacey_string() -> impl Strategy<Value = String> {
    "[a-z ]{0,20}".prop_map(|s| s)
}

// 1. LEN(CONCATENATE(a, b)) == LEN(a) + LEN(b)
#[test]
fn concatenate_len() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(a in ascii_string(), b in ascii_string())| {
        let vars: HashMap<String, Value> = vec![
            ("a".to_string(), Value::Text(a.clone())),
            ("b".to_string(), Value::Text(b.clone())),
        ].into_iter().collect();
        let len_ab = evaluate("=LEN(CONCATENATE(a,b))", &vars);
        let len_a = evaluate("=LEN(a)", &vars);
        let len_b = evaluate("=LEN(b)", &vars);
        if let (Value::Number(total), Value::Number(la), Value::Number(lb)) = (len_ab, len_a, len_b) {
            prop_assert_eq!(total, la + lb);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [a-z]{{0,20}}, b ∈ [a-z]{{0,20}})");
}

// 2. TRIM is idempotent: TRIM(s) == TRIM(TRIM(s))
#[test]
fn trim_idempotent() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in spacey_string())| {
        let vars: HashMap<String, Value> = vec![
            ("s".to_string(), Value::Text(s.clone())),
        ].into_iter().collect();
        let trimmed = evaluate("=TRIM(s)", &vars);
        // Trimming an already-trimmed lowercase ascii string should give same result
        if let Value::Text(t) = trimmed {
            let vars2: HashMap<String, Value> = vec![
                ("s".to_string(), Value::Text(t.clone())),
            ].into_iter().collect();
            let trimmed2 = evaluate("=TRIM(s)", &vars2);
            prop_assert_eq!(trimmed2, Value::Text(t));
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z ]{{0,20}})");
}

// 3. UPPER is idempotent: UPPER(UPPER(s)) == UPPER(s)
#[test]
fn upper_idempotent() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in ascii_string())| {
        let vars: HashMap<String, Value> = vec![
            ("s".to_string(), Value::Text(s.clone())),
        ].into_iter().collect();
        let upper1 = evaluate("=UPPER(s)", &vars);
        if let Value::Text(u) = upper1 {
            let vars2: HashMap<String, Value> = vec![
                ("s".to_string(), Value::Text(u.clone())),
            ].into_iter().collect();
            let upper2 = evaluate("=UPPER(s)", &vars2);
            prop_assert_eq!(upper2, Value::Text(u));
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z]{{0,20}})");
}

// 4. LOWER is idempotent: LOWER(LOWER(s)) == LOWER(s)
#[test]
fn lower_idempotent() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in ascii_string())| {
        let vars: HashMap<String, Value> = vec![
            ("s".to_string(), Value::Text(s.clone())),
        ].into_iter().collect();
        let lower1 = evaluate("=LOWER(s)", &vars);
        if let Value::Text(l) = lower1 {
            let vars2: HashMap<String, Value> = vec![
                ("s".to_string(), Value::Text(l.clone())),
            ].into_iter().collect();
            let lower2 = evaluate("=LOWER(s)", &vars2);
            prop_assert_eq!(lower2, Value::Text(l));
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z]{{0,20}})");
}

// LEN is non-negative for any string
#[test]
fn len_non_negative() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in ascii_string())| {
        let vars: HashMap<String, Value> = [(
            "s".to_string(), Value::Text(s.clone()),
        )].into_iter().collect();
        let result = evaluate("=LEN(s)", &vars);
        if let Value::Number(n) = result {
            prop_assert!(n >= 0.0, "LEN returned negative for {:?}", s);
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z]{{0,20}})");
}

// CONCATENATE length: LEN(CONCATENATE(a, b)) == LEN(a) + LEN(b)
#[test]
fn concatenate_preserves_total_length() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(a in ascii_string(), b in ascii_string())| {
        let vars: HashMap<String, Value> = [
            ("a".to_string(), Value::Text(a.clone())),
            ("b".to_string(), Value::Text(b.clone())),
        ].into_iter().collect();
        let ab_len = evaluate("=LEN(CONCATENATE(a,b))", &vars);
        let a_len  = evaluate("=LEN(a)", &vars);
        let b_len  = evaluate("=LEN(b)", &vars);
        if let (Value::Number(total), Value::Number(la), Value::Number(lb)) = (ab_len, a_len, b_len) {
            prop_assert_eq!(total, la + lb,
                "LEN(CONCATENATE({:?},{:?})) != LEN(a)+LEN(b)", a, b);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [a-z]{{0,20}}, b ∈ [a-z]{{0,20}})");
}

// 5. LEFT(s, LEN(s)) == s  (taking all characters returns the full string)
#[test]
fn left_full_len_is_identity() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in ascii_string())| {
        let len = s.len() as f64;
        let text_var: HashMap<String, Value> = vec![
            ("s".to_string(), Value::Text(s.clone())),
            ("n".to_string(), Value::Number(len)),
        ].into_iter().collect();
        let result = evaluate("=LEFT(s, n)", &text_var);
        prop_assert_eq!(result, Value::Text(s));
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z]{{0,20}})");
}

// Smoke test for text helpers
#[test]
fn concatenate_sanity() {
    let result = run_text_vars("=CONCATENATE(a,b)", vec![("a", "hello"), ("b", " world")]);
    assert_eq!(result, Value::Text("hello world".to_string()));
}
