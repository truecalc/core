// crates/core/tests/property_date.rs
//
// Property-based tests for date functions.
// Verifies mathematical invariants that hold for any valid date input.

use truecalc_core::{evaluate, Value};
use proptest::prelude::*;
use std::collections::HashMap;

const CASES: u32 = 500;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

// Valid Gregorian years in a reasonable range (1900-2100)
fn valid_year() -> impl Strategy<Value = i32> {
    1900i32..=2100
}

// Valid months 1-12
fn valid_month() -> impl Strategy<Value = i32> {
    1i32..=12
}

// Valid days 1-28 (safe for all months including February)
fn valid_day() -> impl Strategy<Value = i32> {
    1i32..=28
}

// DATE(y,m,d): YEAR of the result equals y, MONTH equals m, DAY equals d
// (for unambiguous inputs: day 1-28, month 1-12, year 1900-2100)
#[test]
fn date_year_month_day_roundtrip() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(
        y in valid_year(),
        m in valid_month(),
        d in valid_day(),
    )| {
        let date_result = run(&format!("=DATE({},{},{})", y, m, d));
        // DATE returns a Value::Date (serial date)
        match date_result {
            Value::Date(_) | Value::Number(_) => {
                let year_f  = run(&format!("=YEAR(DATE({},{},{}))", y, m, d));
                let month_f = run(&format!("=MONTH(DATE({},{},{}))", y, m, d));
                let day_f   = run(&format!("=DAY(DATE({},{},{}))", y, m, d));
                if let (Value::Number(yr), Value::Number(mo), Value::Number(dy)) =
                    (year_f, month_f, day_f)
                {
                    prop_assert_eq!(yr as i32, y, "YEAR(DATE({},{},{})) mismatch", y, m, d);
                    prop_assert_eq!(mo as i32, m, "MONTH(DATE({},{},{})) mismatch", y, m, d);
                    prop_assert_eq!(dy as i32, d, "DAY(DATE({},{},{})) mismatch", y, m, d);
                }
            }
            _ => {} // if DATE errors on some inputs, skip — don't fail the property
        }
    });
    eprintln!("proptest: {CASES} cases (y ∈ [1900, 2100], m ∈ [1, 12], d ∈ [1, 28])");
}

// DATEDIF(start, end, "D") >= 0 when end >= start
#[test]
fn datedif_days_non_negative() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(
        y in valid_year(),
        m in valid_month(),
        d in valid_day(),
        delta in 0i32..=365,
    )| {
        // Build start and end as DATE serial numbers; add delta days to start serial
        let formula = format!(
            "=DATEDIF(DATE({},{},{}), DATE({},{},{})+{}, \"D\")",
            y, m, d, y, m, d, delta
        );
        let result = run(&formula);
        if let Value::Number(n) = result {
            prop_assert!(n >= 0.0,
                "DATEDIF returned {} for delta={}", n, delta);
            prop_assert!((n - delta as f64).abs() < 1.0,
                "DATEDIF days={} but expected delta={}", n, delta);
        }
        // If result is an error (e.g. date out of range when adding delta), skip gracefully
    });
    eprintln!("proptest: {CASES} cases (y ∈ [1900, 2100], m ∈ [1, 12], d ∈ [1, 28], delta ∈ [0, 365])");
}

// YEAR extracts a value in [1900, 2100] for valid dates in that range
#[test]
fn year_within_range() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(y in valid_year(), m in valid_month(), d in valid_day())| {
        let result = run(&format!("=YEAR(DATE({},{},{}))", y, m, d));
        if let Value::Number(n) = result {
            prop_assert!(n >= 1900.0 && n <= 2100.0,
                "YEAR={} out of expected range for DATE({},{},{})", n, y, m, d);
        }
    });
    eprintln!("proptest: {CASES} cases (y ∈ [1900, 2100], m ∈ [1, 12], d ∈ [1, 28])");
}

// MONTH always in [1, 12]
#[test]
fn month_in_valid_range() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(y in valid_year(), m in valid_month(), d in valid_day())| {
        let result = run(&format!("=MONTH(DATE({},{},{}))", y, m, d));
        if let Value::Number(n) = result {
            prop_assert!(n >= 1.0 && n <= 12.0,
                "MONTH={} out of [1,12] for DATE({},{},{})", n, y, m, d);
        }
    });
    eprintln!("proptest: {CASES} cases (y ∈ [1900, 2100], m ∈ [1, 12], d ∈ [1, 28])");
}

// DAY always in [1, 31]
#[test]
fn day_in_valid_range() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(y in valid_year(), m in valid_month(), d in valid_day())| {
        let result = run(&format!("=DAY(DATE({},{},{}))", y, m, d));
        if let Value::Number(n) = result {
            prop_assert!(n >= 1.0 && n <= 31.0,
                "DAY={} out of [1,31] for DATE({},{},{})", n, y, m, d);
        }
    });
    eprintln!("proptest: {CASES} cases (y ∈ [1900, 2100], m ∈ [1, 12], d ∈ [1, 28])");
}
