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
