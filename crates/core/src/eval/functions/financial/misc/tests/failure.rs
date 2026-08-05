use super::super::*;
use crate::types::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// MIRR failures
// ---------------------------------------------------------------------------

#[test]
fn mirr_wrong_arg_count() {
    // Wrong number of args → #N/A
    let args = [
        Value::Array(vec![Value::Number(-1000.0), Value::Number(300.0)]),
        Value::Number(0.1),
    ];
    assert_eq!(mirr_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn mirr_too_many_args() {
    let args = [
        Value::Array(vec![Value::Number(-1000.0), Value::Number(300.0)]),
        Value::Number(0.1),
        Value::Number(0.12),
        Value::Number(0.0),
    ];
    assert_eq!(mirr_fn(&args), Value::Error(ErrorKind::NA));
}

// ---------------------------------------------------------------------------
// IPMT failures
// ---------------------------------------------------------------------------

#[test]
fn ipmt_too_few_args() {
    let args = [Value::Number(0.1), Value::Number(1.0), Value::Number(12.0)];
    assert_eq!(ipmt_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn ipmt_period_out_of_range() {
    // per > nper → #NUM!
    let args = [
        Value::Number(0.1),
        Value::Number(13.0), // period 13 when nper=12
        Value::Number(12.0),
        Value::Number(10000.0),
    ];
    assert_eq!(ipmt_fn(&args), Value::Error(ErrorKind::Num));
}

// ---------------------------------------------------------------------------
// SLN failures
// ---------------------------------------------------------------------------

#[test]
fn sln_zero_life() {
    // life=0 → division by zero → #DIV/0!
    let args = [
        Value::Number(10000.0),
        Value::Number(1000.0),
        Value::Number(0.0),
    ];
    assert_eq!(sln_fn(&args), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn sln_too_few_args() {
    let args = [Value::Number(10000.0), Value::Number(1000.0)];
    assert_eq!(sln_fn(&args), Value::Error(ErrorKind::NA));
}

// ---------------------------------------------------------------------------
// FVSCHEDULE failures
// ---------------------------------------------------------------------------

#[test]
fn fvschedule_too_few_args() {
    let args = [Value::Number(1000.0)];
    assert_eq!(fvschedule_fn(&args), Value::Error(ErrorKind::NA));
}

// ---------------------------------------------------------------------------
// PPMT failures
// ---------------------------------------------------------------------------

#[test]
fn ppmt_too_few_args() {
    let args = [Value::Number(0.1), Value::Number(1.0), Value::Number(12.0)];
    assert_eq!(ppmt_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn ppmt_too_many_args() {
    let args = [
        Value::Number(0.1), Value::Number(1.0), Value::Number(12.0),
        Value::Number(10000.0), Value::Number(0.0), Value::Number(0.0), Value::Number(0.0),
    ];
    assert_eq!(ppmt_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn ppmt_period_zero_returns_num() {
    let args = [
        Value::Number(0.1 / 12.0),
        Value::Number(0.0),
        Value::Number(12.0),
        Value::Number(10000.0),
    ];
    assert_eq!(ppmt_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn ppmt_period_exceeds_nper_returns_num() {
    let args = [
        Value::Number(0.1 / 12.0),
        Value::Number(13.0),
        Value::Number(12.0),
        Value::Number(10000.0),
    ];
    assert_eq!(ppmt_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn ppmt_non_numeric_pv_returns_value_error() {
    let args = [
        Value::Number(0.1 / 12.0),
        Value::Number(1.0),
        Value::Number(12.0),
        Value::Text("ten thousand".to_string()),
    ];
    assert_eq!(ppmt_fn(&args), Value::Error(ErrorKind::Value));
}

// ---------------------------------------------------------------------------
// XIRR failures
// ---------------------------------------------------------------------------

#[test]
fn xirr_too_few_args() {
    let args = [Value::Array(vec![Value::Number(-1000.0), Value::Number(1100.0)])];
    assert_eq!(xirr_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn xirr_single_value_returns_num() {
    // Fewer than 2 values → #NUM!
    let args = [
        Value::Array(vec![Value::Number(-1000.0)]),
        Value::Array(vec![Value::Number(44927.0)]),
    ];
    assert_eq!(xirr_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn xirr_mismatched_lengths_returns_num() {
    let args = [
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1100.0), Value::Number(200.0)]),
        Value::Array(vec![Value::Number(44927.0), Value::Number(45292.0)]),
    ];
    assert_eq!(xirr_fn(&args), Value::Error(ErrorKind::Num));
}

// ---------------------------------------------------------------------------
// XNPV failures
// ---------------------------------------------------------------------------

#[test]
fn xnpv_too_few_args() {
    let args = [
        Value::Number(0.1),
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1100.0)]),
    ];
    assert_eq!(xnpv_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn xnpv_mismatched_lengths() {
    // values and dates have different lengths → #NUM!
    let args = [
        Value::Number(0.1),
        Value::Array(vec![Value::Number(-1000.0), Value::Number(1100.0)]),
        Value::Array(vec![Value::Number(44927.0)]),
    ];
    assert_eq!(xnpv_fn(&args), Value::Error(ErrorKind::Num));
}
