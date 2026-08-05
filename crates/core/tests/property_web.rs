use proptest::prelude::*;
use truecalc_core::Value;
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
    "[A-Za-z0-9\\-_.~]{0,30}"
}

fn ascii_with_spaces() -> impl Strategy<Value = String> {
    // Must include at least one space so the %20 expansion path is exercised
    "[a-z]{0,10} [a-z]{0,10}"
}

fn pure_alpha() -> impl Strategy<Value = String> {
    "[a-zA-Z]{1,20}"
}

fn ascii_string() -> impl Strategy<Value = String> {
    "[a-z]{0,20}"
}

fn small_f64() -> impl Strategy<Value = f64> {
    -1e6f64..1e6f64
}

// ── ENCODEURL ────────────────────────────────────────────────────────────────

// Unreserved chars (RFC 3986) are passed through unchanged
#[test]
fn encodeurl_unreserved_chars_unchanged() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in unreserved_string())| {
        let result = run_text("=ENCODEURL(s)", vec![("s", &s)]);
        prop_assert_eq!(result, Value::Text(s));
    });
    eprintln!("proptest: {CASES} cases (s ∈ [A-Za-z0-9\\-_.~]{{0,30}})");
}

// Output length >= input length; space encodes to %20 so > is exercised
#[test]
fn encodeurl_output_length_ge_input() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in ascii_with_spaces())| {
        let len_s = s.len() as f64;
        let result = run_text("=LEN(ENCODEURL(s))", vec![("s", &s)]);
        if let Value::Number(n) = result {
            prop_assert!(n > len_s,
                "expected strict growth for {:?} (space → %20), got len {n}", s);
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z]{{0,10}} [a-z]{{0,10}}, always contains space)");
}

// ENCODEURL is identity on unreserved inputs, so encoding twice == encoding once
#[test]
fn encodeurl_idempotent_on_unreserved() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in unreserved_string())| {
        // Unreserved chars are unchanged, so ENCODEURL(s) == s and re-encoding gives s again
        let first  = run_text("=ENCODEURL(s)", vec![("s", &s)]);
        if let Value::Text(ref encoded) = first {
            let second = run_text("=ENCODEURL(s)", vec![("s", encoded)]);
            prop_assert_eq!(second, first);
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [A-Za-z0-9\\-_.~]{{0,30}})");
}

// ── ISURL ────────────────────────────────────────────────────────────────────

// Non-text values always return FALSE
#[test]
fn isurl_false_for_numbers() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_num("=ISURL(x)", x);
        prop_assert_eq!(result, Value::Bool(false));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// Strings with a valid https:// scheme are always TRUE (scheme branch)
#[test]
fn isurl_true_for_https_urls() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(host in "[a-z]{1,10}")| {
        let url = format!("https://{host}.com");
        let result = run_text("=ISURL(s)", vec![("s", &url)]);
        prop_assert_eq!(result, Value::Bool(true), "expected ISURL TRUE for {:?}", url);
    });
    eprintln!("proptest: {CASES} cases (url = https://<host>.com)");
}

// Strings with no dot and no :// scheme are always FALSE (negative boundary)
#[test]
fn isurl_false_for_no_dot_no_scheme() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in pure_alpha())| {
        let result = run_text("=ISURL(s)", vec![("s", &s)]);
        prop_assert_eq!(result, Value::Bool(false),
            "expected ISURL FALSE for {:?} (no dot, no scheme)", s);
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-zA-Z]{{1,20}}, no dot or ://)");
}

// ── HYPERLINK ────────────────────────────────────────────────────────────────

// Single-arg form returns the url unchanged
#[test]
fn hyperlink_no_label_returns_url() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(url in ascii_string())| {
        let result = run_text("=HYPERLINK(url)", vec![("url", &url)]);
        prop_assert_eq!(result, Value::Text(url));
    });
    eprintln!("proptest: {CASES} cases (url ∈ [a-z]{{0,20}})");
}

// Two-arg form returns the label unchanged (text label)
#[test]
fn hyperlink_with_label_returns_label() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(url in ascii_string(), label in ascii_string())| {
        let result = run_text("=HYPERLINK(url, label)", vec![("url", &url), ("label", &label)]);
        prop_assert_eq!(result, Value::Text(label));
    });
    eprintln!("proptest: {CASES} cases (url ∈ [a-z]{{0,20}}, label ∈ [a-z]{{0,20}})");
}

// HYPERLINK passes numeric labels through without coercion
#[test]
fn hyperlink_numeric_label_passes_through() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let vars: HashMap<String, Value> = [
            ("url".to_string(), Value::Text("https://example.com".to_string())),
            ("n".to_string(), Value::Number(x)),
        ].into_iter().collect();
        let result = evaluate("=HYPERLINK(url, n)", &vars);
        prop_assert_eq!(result, Value::Number(x));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// ── Sanity checks ─────────────────────────────────────────────────────────────

#[test]
fn sanity_encodeurl() {
    let result = run_text("=ENCODEURL(s)", vec![("s", "hello world")]);
    assert_eq!(result, Value::Text("hello%20world".to_string()));
}

#[test]
fn sanity_isurl() {
    assert_eq!(run_text("=ISURL(s)", vec![("s", "https://example.com")]), Value::Bool(true));
    assert_eq!(run_text("=ISURL(s)", vec![("s", "not-a-url")]),           Value::Bool(false));
    assert_eq!(run_text("=ISURL(s)", vec![("s", "example.com")]),         Value::Bool(true));
}

#[test]
fn sanity_hyperlink() {
    assert_eq!(
        run_text("=HYPERLINK(url, label)", vec![("url", "https://x.com"), ("label", "Click")]),
        Value::Text("Click".to_string())
    );
}

/// Engine-explicit shim: the free `evaluate` is deprecated in favor of
/// `Engine::sheets().evaluate` (ADR 2026-04-27).
fn evaluate(formula: &str, variables: &std::collections::HashMap<String, Value>) -> Value {
    truecalc_core::Engine::sheets().evaluate(formula, variables)
}
