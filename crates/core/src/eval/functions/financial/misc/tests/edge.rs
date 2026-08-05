use super::super::*;
use crate::types::{ErrorKind, Value};

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

// ---------------------------------------------------------------------------
// IPMT edge cases
// ---------------------------------------------------------------------------

#[test]
fn ipmt_zero_rate_returns_zero() {
    // No interest when rate = 0
    let args = [
        Value::Number(0.0),
        Value::Number(3.0),
        Value::Number(12.0),
        Value::Number(12000.0),
    ];
    assert_eq!(ipmt_fn(&args), Value::Number(0.0));
}

#[test]
fn ipmt_type1_period1_is_zero() {
    // With type=1 (annuity-due) the first payment happens before any interest accrues
    let args = [
        Value::Number(0.1 / 12.0),
        Value::Number(1.0),
        Value::Number(12.0),
        Value::Number(10000.0),
        Value::Number(0.0),
        Value::Number(1.0),
    ];
    assert_eq!(ipmt_fn(&args), Value::Number(0.0));
}

#[test]
fn ipmt_interest_decreases_over_time() {
    // Period 1 has more interest than period 12 (loan amortisation)
    let base = [
        Value::Number(0.1 / 12.0),
        Value::Number(0.0), // placeholder
        Value::Number(12.0),
        Value::Number(10000.0),
    ];
    let mut args1 = base.clone(); args1[1] = Value::Number(1.0);
    let mut args12 = base.clone(); args12[1] = Value::Number(12.0);
    let i1 = if let Value::Number(n) = ipmt_fn(&args1) { n } else { panic!() };
    let i12 = if let Value::Number(n) = ipmt_fn(&args12) { n } else { panic!() };
    // Both negative; period-1 interest is more negative (larger magnitude)
    assert!(i1 < i12, "expected |ipmt(1)| > |ipmt(12)|, got {} vs {}", i1, i12);
}

// ---------------------------------------------------------------------------
// PPMT edge cases
// ---------------------------------------------------------------------------

#[test]
fn ppmt_period1_type1_equals_full_pmt() {
    // With type=1, IPMT(1)=0 so PPMT(1) = entire PMT payment
    let args = [
        Value::Number(0.1 / 12.0),
        Value::Number(1.0),
        Value::Number(12.0),
        Value::Number(10000.0),
        Value::Number(0.0),
        Value::Number(1.0),
    ];
    let ppmt = ppmt_fn(&args);
    // PMT_type1 = PMT_type0 / (1+r) ≈ -879.16 / 1.008333 ≈ -872.27
    assert!(approx(ppmt.clone(), -872.27, 1.0), "got {:?}", ppmt);
    // And IPMT(1, type=1) = 0, so PPMT should equal PMT entirely
    let ipmt = ipmt_fn(&args);
    assert_eq!(ipmt, Value::Number(0.0));
}

#[test]
fn ppmt_principal_increases_over_time() {
    // Later periods have larger principal payments (smaller interest)
    let base = [
        Value::Number(0.1 / 12.0),
        Value::Number(0.0), // placeholder
        Value::Number(12.0),
        Value::Number(10000.0),
    ];
    let mut args1 = base.clone(); args1[1] = Value::Number(1.0);
    let mut args12 = base.clone(); args12[1] = Value::Number(12.0);
    let p1 = if let Value::Number(n) = ppmt_fn(&args1) { n } else { panic!() };
    let p12 = if let Value::Number(n) = ppmt_fn(&args12) { n } else { panic!() };
    // Both negative; last period has larger magnitude principal
    assert!(p12 < p1, "expected |ppmt(12)| > |ppmt(1)|, got {} vs {}", p12, p1);
}

// ---------------------------------------------------------------------------
// XIRR edge cases
// ---------------------------------------------------------------------------

#[test]
fn xirr_all_positive_returns_num() {
    let args = [
        Value::Array(vec![Value::Number(100.0), Value::Number(200.0)]),
        Value::Array(vec![Value::Number(44927.0), Value::Number(45292.0)]),
    ];
    assert_eq!(xirr_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn xirr_all_negative_returns_num() {
    let args = [
        Value::Array(vec![Value::Number(-100.0), Value::Number(-200.0)]),
        Value::Array(vec![Value::Number(44927.0), Value::Number(45292.0)]),
    ];
    assert_eq!(xirr_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn xirr_mismatched_array_lengths_returns_num() {
    let args = [
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1100.0)]),
        Value::Array(vec![Value::Number(44927.0)]),
    ];
    assert_eq!(xirr_fn(&args), Value::Error(ErrorKind::Num));
}

// ---------------------------------------------------------------------------
// FVSCHEDULE edge cases
// ---------------------------------------------------------------------------

#[test]
fn fvschedule_empty_rates_returns_principal() {
    // FVSCHEDULE(1000, []) = 1000 (no compounding)
    let args = [Value::Number(1000.0), Value::Array(vec![])];
    assert!(approx(fvschedule_fn(&args), 1000.0, 1e-9));
}

#[test]
fn sln_equal_cost_and_salvage_returns_zero() {
    // SLN(5000, 5000, 5) = 0 (no depreciation)
    let args = [Value::Number(5000.0), Value::Number(5000.0), Value::Number(5.0)];
    assert!(approx(sln_fn(&args), 0.0, 1e-9));
}

#[test]
fn xnpv_zero_rate_sums_cashflows() {
    // At rate=0, XNPV = sum of all cashflows = -1000 + 1100 = 100
    let args = [
        Value::Number(0.0),
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1100.0)]),
        Value::Array(vec![Value::Number(44927.0), Value::Number(45292.0)]),
    ];
    assert!(approx(xnpv_fn(&args), 100.0, 1e-6));
}
