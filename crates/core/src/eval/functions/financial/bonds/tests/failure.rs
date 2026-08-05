use super::super::*;
use crate::types::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// PRICEDISC failures
// ---------------------------------------------------------------------------

#[test]
fn pricedisc_too_few_args() {
    let args = [Value::Number(44927.0), Value::Number(45108.0), Value::Number(0.05)];
    assert_eq!(pricedisc_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn pricedisc_settlement_after_maturity() {
    // settlement >= maturity → #NUM!
    let args = [
        Value::Number(45108.0), // maturity as settlement
        Value::Number(44927.0), // settlement as maturity
        Value::Number(0.05),
        Value::Number(100.0),
    ];
    assert_eq!(pricedisc_fn(&args), Value::Error(ErrorKind::Num));
}

// ---------------------------------------------------------------------------
// DISC failures
// ---------------------------------------------------------------------------

#[test]
fn disc_too_few_args() {
    let args = [Value::Number(44927.0), Value::Number(45108.0), Value::Number(97.5)];
    assert_eq!(disc_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn disc_zero_redemption() {
    // redemption = 0 → #NUM!
    let args = [
        Value::Number(44927.0),
        Value::Number(45108.0),
        Value::Number(97.5),
        Value::Number(0.0),
    ];
    assert_eq!(disc_fn(&args), Value::Error(ErrorKind::Num));
}

// ---------------------------------------------------------------------------
// TBILLPRICE failures
// ---------------------------------------------------------------------------

#[test]
fn tbillprice_too_few_args() {
    let args = [Value::Number(44927.0), Value::Number(45108.0)];
    assert_eq!(tbillprice_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn tbillprice_zero_discount() {
    // discount <= 0 → #NUM!
    let args = [
        Value::Number(44927.0),
        Value::Number(45108.0),
        Value::Number(0.0),
    ];
    assert_eq!(tbillprice_fn(&args), Value::Error(ErrorKind::Num));
}

// ---------------------------------------------------------------------------
// COUPDAYS failures
// ---------------------------------------------------------------------------

#[test]
fn coupdays_too_few_args() {
    let args = [Value::Number(44927.0), Value::Number(45658.0)];
    assert_eq!(coupdays_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn coupdays_invalid_frequency() {
    // frequency=3 is invalid (only 1, 2, 4 allowed)
    let args = [
        Value::Number(44927.0),
        Value::Number(45658.0),
        Value::Number(3.0),
    ];
    assert_eq!(coupdays_fn(&args), Value::Error(ErrorKind::Num));
}

// ---------------------------------------------------------------------------
// validate_basis
// ---------------------------------------------------------------------------

#[test]
fn basis_5_is_invalid() {
    assert_eq!(validate_basis(5.0), Err(Value::Error(ErrorKind::Num)));
}

#[test]
fn basis_negative_is_invalid() {
    assert_eq!(validate_basis(-1.0), Err(Value::Error(ErrorKind::Num)));
}

// ---------------------------------------------------------------------------
// validate_frequency
// ---------------------------------------------------------------------------

#[test]
fn frequency_3_invalid() {
    assert_eq!(validate_frequency(3.0), Err(Value::Error(ErrorKind::Num)));
}

#[test]
fn frequency_0_invalid() {
    assert_eq!(validate_frequency(0.0), Err(Value::Error(ErrorKind::Num)));
}

// ---------------------------------------------------------------------------
// PRICE error paths
// ---------------------------------------------------------------------------

#[test]
fn price_settlement_after_maturity_returns_num() {
    // settlement=46022 (2025-12-31) > maturity=45292 (2024-01-01)
    let args = [
        Value::Number(46022.0),
        Value::Number(45292.0),
        Value::Number(0.05),
        Value::Number(0.05),
        Value::Number(100.0),
        Value::Number(2.0),
    ];
    assert_eq!(price_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn price_invalid_frequency_returns_num() {
    // frequency=3 is invalid
    let args = [
        Value::Number(45292.0),
        Value::Number(46022.0),
        Value::Number(0.05),
        Value::Number(0.05),
        Value::Number(100.0),
        Value::Number(3.0),
    ];
    assert_eq!(price_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn price_invalid_basis_returns_num() {
    // basis=5 is invalid
    let args = [
        Value::Number(45292.0),
        Value::Number(46022.0),
        Value::Number(0.05),
        Value::Number(0.05),
        Value::Number(100.0),
        Value::Number(2.0),
        Value::Number(5.0),
    ];
    assert_eq!(price_fn(&args), Value::Error(ErrorKind::Num));
}

// ---------------------------------------------------------------------------
// YIELD error paths
// ---------------------------------------------------------------------------

#[test]
fn yield_settlement_after_maturity_returns_num() {
    // settlement=46022 > maturity=45292
    let args = [
        Value::Number(46022.0),
        Value::Number(45292.0),
        Value::Number(0.05),
        Value::Number(100.0),
        Value::Number(100.0),
        Value::Number(2.0),
    ];
    assert_eq!(yield_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn yield_invalid_frequency_returns_num() {
    // frequency=3 is invalid
    let args = [
        Value::Number(45292.0),
        Value::Number(46022.0),
        Value::Number(0.05),
        Value::Number(100.0),
        Value::Number(100.0),
        Value::Number(3.0),
    ];
    assert_eq!(yield_fn(&args), Value::Error(ErrorKind::Num));
}

// ---------------------------------------------------------------------------
// ACCRINT error paths
// ---------------------------------------------------------------------------

#[test]
fn accrint_invalid_frequency_returns_num() {
    // frequency=3 is invalid; args: issue, first_interest, settlement, rate, par, frequency
    let args = [
        Value::Number(45292.0),
        Value::Number(45383.0),
        Value::Number(45474.0),
        Value::Number(0.05),
        Value::Number(1000.0),
        Value::Number(3.0),
    ];
    assert_eq!(accrint_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn accrint_invalid_basis_returns_num() {
    // basis=9 is invalid
    let args = [
        Value::Number(45292.0),
        Value::Number(45383.0),
        Value::Number(45474.0),
        Value::Number(0.05),
        Value::Number(1000.0),
        Value::Number(2.0),
        Value::Number(9.0),
    ];
    assert_eq!(accrint_fn(&args), Value::Error(ErrorKind::Num));
}
