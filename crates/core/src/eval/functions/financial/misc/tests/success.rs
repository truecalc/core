use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

// ---------------------------------------------------------------------------
// IPMT
// ---------------------------------------------------------------------------

#[test]
fn ipmt_first_period_interest() {
    // IPMT(0.1/12, 1, 12, 10000)
    // Balance at start of period 1 = 10000
    // Interest = -(10000 * 0.1/12) = -83.3333
    let args = [
        Value::Number(0.1 / 12.0),
        Value::Number(1.0),
        Value::Number(12.0),
        Value::Number(10000.0),
    ];
    let result = ipmt_fn(&args);
    assert!(approx(result.clone(), -83.3333, 1e-4), "got {:?}", result);
}

#[test]
fn ipmt_result_is_negative() {
    // Interest payments on a loan should be negative (outflows)
    let args = [
        Value::Number(0.1 / 12.0),
        Value::Number(1.0),
        Value::Number(12.0),
        Value::Number(10000.0),
    ];
    if let Value::Number(n) = ipmt_fn(&args) {
        assert!(n < 0.0, "interest payment should be negative, got {}", n);
    } else {
        panic!("expected Number");
    }
}

// ---------------------------------------------------------------------------
// SLN
// ---------------------------------------------------------------------------

#[test]
fn sln_basic() {
    // SLN(10000, 1000, 5) = (10000 - 1000) / 5 = 1800
    let args = [
        Value::Number(10000.0),
        Value::Number(1000.0),
        Value::Number(5.0),
    ];
    assert!(approx(sln_fn(&args), 1800.0, 1e-9));
}

#[test]
fn sln_zero_salvage() {
    // SLN(6000, 0, 3) = 2000
    let args = [
        Value::Number(6000.0),
        Value::Number(0.0),
        Value::Number(3.0),
    ];
    assert!(approx(sln_fn(&args), 2000.0, 1e-9));
}

// ---------------------------------------------------------------------------
// FVSCHEDULE
// ---------------------------------------------------------------------------

#[test]
fn fvschedule_two_rates() {
    // FVSCHEDULE(1000, [0.05, 0.05]) = 1000 * 1.05 * 1.05 = 1102.5
    let args = [
        Value::Number(1000.0),
        Value::Array(vec![Value::Number(0.05), Value::Number(0.05)]),
    ];
    assert!(approx(fvschedule_fn(&args), 1102.5, 1e-4));
}

#[test]
fn fvschedule_single_rate() {
    // FVSCHEDULE(500, [0.1]) = 550
    let args = [
        Value::Number(500.0),
        Value::Array(vec![Value::Number(0.1)]),
    ];
    assert!(approx(fvschedule_fn(&args), 550.0, 1e-9));
}

// ---------------------------------------------------------------------------
// MIRR
// ---------------------------------------------------------------------------

#[test]
fn mirr_positive_return() {
    // MIRR([-1000, 300, 400, 500], 0.1, 0.12)
    // FV of positives at 12%: 300*(1.12^2) + 400*(1.12^1) + 500*(1.12^0)
    //   = 376.32 + 448 + 500 = 1324.32
    // NPV of negatives at 10%: -1000
    // MIRR = (1324.32 / 1000)^(1/3) - 1 ≈ 0.09816
    let args = [
        Value::Array(vec![
            Value::Number(-1000.0),
            Value::Number(300.0),
            Value::Number(400.0),
            Value::Number(500.0),
        ]),
        Value::Number(0.1),
        Value::Number(0.12),
    ];
    let result = mirr_fn(&args);
    assert!(approx(result.clone(), 0.09816, 1e-4), "got {:?}", result);
}

// ---------------------------------------------------------------------------
// PPMT
// ---------------------------------------------------------------------------

#[test]
fn ppmt_period_1_interest_and_principal() {
    // PPMT(10%/12, 1, 12, 10000): period 1 principal on a 12-month 10% loan
    // PMT ≈ -879.16, IPMT(1) = -(10000 * 0.1/12) = -83.33, PPMT = -795.83
    let args = [
        Value::Number(0.1 / 12.0),
        Value::Number(1.0),
        Value::Number(12.0),
        Value::Number(10000.0),
    ];
    let result = ppmt_fn(&args);
    assert!(approx(result.clone(), -795.8255, 1e-3), "got {:?}", result);
}

#[test]
fn ppmt_zero_rate_equal_payments() {
    // PPMT(0, per, 12, 12000) = -1000 regardless of period (zero interest)
    for per in [1.0, 6.0, 12.0] {
        let args = [
            Value::Number(0.0),
            Value::Number(per),
            Value::Number(12.0),
            Value::Number(12000.0),
        ];
        let result = ppmt_fn(&args);
        assert!(approx(result.clone(), -1000.0, 1e-9), "per={} got {:?}", per, result);
    }
}

#[test]
fn ppmt_plus_ipmt_equals_pmt() {
    // Invariant: PPMT(r, k, n, pv) + IPMT(r, k, n, pv) = PMT(r, n, pv) for any k
    let rate = Value::Number(0.1 / 12.0);
    let nper = Value::Number(12.0);
    let pv = Value::Number(10000.0);
    for per in [1.0_f64, 5.0, 12.0] {
        let period = Value::Number(per);
        let pmt_args = [rate.clone(), period.clone(), nper.clone(), pv.clone()];
        let ppmt = match ppmt_fn(&pmt_args) { Value::Number(n) => n, v => panic!("ppmt@per={}: {:?}", per, v) };
        let ipmt = match ipmt_fn(&pmt_args) { Value::Number(n) => n, v => panic!("ipmt@per={}: {:?}", per, v) };
        // PMT for this loan ≈ -879.16
        let total = ppmt + ipmt;
        assert!((total - (-879.1588)).abs() < 1e-3, "per={} ppmt+ipmt={} want -879.1588", per, total);
    }
}

// ---------------------------------------------------------------------------
// XIRR
// ---------------------------------------------------------------------------

#[test]
fn xirr_exact_one_year_ten_percent() {
    // XIRR([-1000, 1100], [2023-01-01, 2024-01-01]) = 10%
    // 44927 = Excel serial for 2023-01-01, 45292 = 2024-01-01 (365 days apart)
    let args = [
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1100.0)]),
        Value::Array(vec![Value::Number(44927.0), Value::Number(45292.0)]),
    ];
    let result = xirr_fn(&args);
    assert!(approx(result.clone(), 0.1, 1e-6), "got {:?}", result);
}

#[test]
fn xirr_two_year_ten_percent() {
    // XIRR([-1000, 1210], [2023-01-01, 2025-01-01]) = 10%
    // 45657 = 44927 + 730 (two 365-day years)
    let args = [
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1210.0)]),
        Value::Array(vec![Value::Number(44927.0), Value::Number(45657.0)]),
    ];
    let result = xirr_fn(&args);
    assert!(approx(result.clone(), 0.1, 1e-6), "got {:?}", result);
}

#[test]
fn xirr_with_explicit_guess() {
    // Same as xirr_exact_one_year_ten_percent but with guess=0.05
    let args = [
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1100.0)]),
        Value::Array(vec![Value::Number(44927.0), Value::Number(45292.0)]),
        Value::Number(0.05),
    ];
    let result = xirr_fn(&args);
    assert!(approx(result.clone(), 0.1, 1e-6), "got {:?}", result);
}

// ---------------------------------------------------------------------------
// XNPV
// ---------------------------------------------------------------------------

#[test]
fn xnpv_break_even() {
    // XNPV(0.1, [-1000, 1100], [2023-01-01, 2024-01-01])
    // date0 = 44927 (2023-01-01), date1 = 45292 (2024-01-01)
    // t1 = 365/365 = 1.0 year
    // NPV = -1000 + 1100 / 1.1^1 = 0
    let args = [
        Value::Number(0.1),
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1100.0)]),
        Value::Array(vec![Value::Number(44927.0), Value::Number(45292.0)]),
    ];
    assert!(approx(xnpv_fn(&args), 0.0, 1e-4));
}

#[test]
fn xnpv_positive_npv() {
    // XNPV(0.1, [-1000, 1200], [date0, date1_year_later])
    // NPV = -1000 + 1200/1.1 = -1000 + 1090.909... = 90.909
    let args = [
        Value::Number(0.1),
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1200.0)]),
        Value::Array(vec![Value::Number(44927.0), Value::Number(45292.0)]),
    ];
    assert!(approx(xnpv_fn(&args), 90.909, 1e-3));
}
