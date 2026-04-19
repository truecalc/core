use proptest::prelude::*;
use truecalc_core::{evaluate, Value};
use std::collections::HashMap;

const CASES: u32 = 500;

fn run_text(formula: &str, vars: Vec<(&str, &str)>) -> Value {
    let map: HashMap<String, Value> = vars
        .into_iter()
        .map(|(k, v)| (k.to_string(), Value::Text(v.to_string())))
        .collect();
    evaluate(formula, &map)
}

fn run_num(formula: &str, x: f64) -> Value {
    let map = [("x".to_string(), Value::Number(x))].into_iter().collect();
    evaluate(formula, &map)
}

fn unreserved_string() -> impl Strategy<Value = String> {
    "[A-Za-z0-9\\-_.~]{0,30}".prop_map(|s| s)
}

fn ascii_string() -> impl Strategy<Value = String> {
    "[a-z]{0,20}".prop_map(|s| s)
}

fn small_f64() -> impl Strategy<Value = f64> {
    -1e6f64..1e6f64
}

// ENCODEURL: unreserved chars are left unchanged
#[test]
fn encodeurl_unreserved_chars_unchanged() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in unreserved_string())| {
        let result = run_text("=ENCODEURL(s)", vec![("s", &s)]);
        prop_assert_eq!(result, Value::Text(s));
    });
    eprintln!("proptest: {CASES} cases (s ∈ [A-Za-z0-9\\-_.~]{{0,30}})");
}

// ENCODEURL: output length >= input length (encoding can only grow)
#[test]
fn encodeurl_output_length_ge_input() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in ascii_string())| {
        let len_s = s.len() as f64;
        let result = run_text("=LEN(ENCODEURL(s))", vec![("s", &s)]);
        if let Value::Number(n) = result {
            prop_assert!(n >= len_s, "ENCODEURL output shorter than input for {:?}", s);
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z]{{0,20}})");
}

// ENCODEURL: idempotent on unreserved strings (encoding them again yields same result)
#[test]
fn encodeurl_idempotent_on_unreserved() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in unreserved_string())| {
        let once = run_text("=ENCODEURL(s)", vec![("s", &s)]);
        if let Value::Text(encoded) = once {
            let twice = run_text("=ENCODEURL(s)", vec![("s", &encoded)]);
            prop_assert_eq!(twice, Value::Text(encoded));
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [A-Za-z0-9\\-_.~]{{0,30}})");
}

// ISURL: always FALSE for numeric inputs
#[test]
fn isurl_false_for_numbers() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_num("=ISURL(x)", x);
        prop_assert_eq!(result, Value::Bool(false));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// ISURL: always TRUE for https:// URLs
#[test]
fn isurl_true_for_https_urls() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(host in "[a-z]{1,10}")| {
        let url = format!("https://{host}.com");
        let result = run_text("=ISURL(s)", vec![("s", &url)]);
        prop_assert_eq!(result, Value::Bool(true), "expected ISURL to be TRUE for {:?}", url);
    });
    eprintln!("proptest: {CASES} cases (url = https://<host>.com)");
}

// HYPERLINK(url) returns url unchanged
#[test]
fn hyperlink_no_label_returns_url() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(url in ascii_string())| {
        let result = run_text("=HYPERLINK(url)", vec![("url", &url)]);
        prop_assert_eq!(result, Value::Text(url));
    });
    eprintln!("proptest: {CASES} cases (url ∈ [a-z]{{0,20}})");
}

// HYPERLINK(url, label) returns label unchanged
#[test]
fn hyperlink_with_label_returns_label() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(url in ascii_string(), label in ascii_string())| {
        let result = run_text("=HYPERLINK(url, label)", vec![("url", &url), ("label", &label)]);
        prop_assert_eq!(result, Value::Text(label));
    });
    eprintln!("proptest: {CASES} cases (url ∈ [a-z]{{0,20}}, label ∈ [a-z]{{0,20}})");
}

// HYPERLINK: LEN(HYPERLINK(url, label)) == LEN(label)
#[test]
fn hyperlink_label_len_matches() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(url in ascii_string(), label in ascii_string())| {
        let label_len = label.len() as f64;
        let result = run_text("=LEN(HYPERLINK(url, label))", vec![("url", &url), ("label", &label)]);
        prop_assert_eq!(result, Value::Number(label_len));
    });
    eprintln!("proptest: {CASES} cases (url ∈ [a-z]{{0,20}}, label ∈ [a-z]{{0,20}})");
}
