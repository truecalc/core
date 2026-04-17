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
