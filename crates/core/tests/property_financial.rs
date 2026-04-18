use proptest::prelude::*;
use ganit_core::{evaluate, Value};
use std::collections::HashMap;

const CASES: u32 = 500;

fn run_vars(formula: &str, vars: Vec<(&str, f64)>) -> Value {
    let map = vars.into_iter().map(|(k, v)| (k.to_string(), Value::Number(v))).collect();
    evaluate(formula, &map)
}

fn small_positive() -> impl Strategy<Value = f64> {
    1.0f64..1e6f64
}

// 1. NPV(0, v1, v2, v3) ≈ v1 + v2 + v3 — zero discount rate = simple sum
#[test]
fn npv_zero_rate_is_sum() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(
        v1 in small_positive(),
        v2 in small_positive(),
        v3 in small_positive()
    )| {
        let result = run_vars("=NPV(0, v1, v2, v3)", vec![("v1", v1), ("v2", v2), ("v3", v3)]);
        let expected = v1 + v2 + v3;
        prop_assert!(matches!(result, Value::Number(_)), "expected Number, got {:?}", result);
        if let Value::Number(n) = result {
            prop_assert!((n - expected).abs() < 1e-6, "NPV(0,...)={} expected={}", n, expected);
        }
    });
    eprintln!("proptest: {CASES} cases (v1, v2, v3 ∈ [1, 1e6])");
}

// 2. PMT sign is opposite to PV sign for standard loans (positive PV -> negative PMT)
#[test]
fn pmt_sign_opposite_to_pv() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(
        rate in 0.001f64..0.2f64,
        nper in 1.0f64..60.0f64,
        pv in 1000.0f64..100_000.0f64
    )| {
        let result = run_vars("=PMT(r, n, p)", vec![("r", rate), ("n", nper), ("p", pv)]);
        prop_assert!(matches!(result, Value::Number(_)), "expected Number, got {:?}", result);
        if let Value::Number(pmt) = result {
            // positive PV (loan amount) should give negative PMT (payment out)
            prop_assert!(pmt < 0.0, "PMT should be negative for positive PV, got {}", pmt);
        }
    });
    eprintln!("proptest: {CASES} cases (rate ∈ [0.001, 0.2], nper ∈ [1, 60], pv ∈ [1000, 100000])");
}

// 3. PMT with negative PV gives positive PMT (savings scenario)
#[test]
fn pmt_sign_positive_for_negative_pv() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(
        rate in 0.001f64..0.2f64,
        nper in 1.0f64..60.0f64,
        pv in 1000.0f64..100_000.0f64
    )| {
        let result = run_vars("=PMT(r, n, p)", vec![("r", rate), ("n", nper), ("p", -pv)]);
        prop_assert!(matches!(result, Value::Number(_)), "expected Number, got {:?}", result);
        if let Value::Number(pmt) = result {
            prop_assert!(pmt > 0.0, "PMT should be positive for negative PV, got {}", pmt);
        }
    });
    eprintln!("proptest: {CASES} cases (rate ∈ [0.001, 0.2], nper ∈ [1, 60], pv ∈ [1000, 100000])");
}

// Sanity checks
#[test]
fn npv_zero_rate_sanity() {
    // NPV(0, 100, 200) should be 100 + 200 = 300
    let vars: HashMap<String, Value> = vec![
        ("v1".to_string(), Value::Number(100.0)),
        ("v2".to_string(), Value::Number(200.0)),
    ].into_iter().collect();
    let result = evaluate("=NPV(0, v1, v2)", &vars);
    assert_eq!(result, Value::Number(300.0));
}
