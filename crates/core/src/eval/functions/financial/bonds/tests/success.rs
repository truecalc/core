use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

// Serial dates (base: 1899-12-30):
//   2023-01-01 = 44927
//   2023-07-01 = 45108  (181 days later)
//   2025-01-01 = 45658

// ---------------------------------------------------------------------------
// PRICEDISC
// ---------------------------------------------------------------------------

#[test]
fn pricedisc_6month_tbill() {
    // PRICEDISC(settlement=2023-01-01, maturity=2023-07-01, discount=5%, redemption=100, basis=2)
    // yearfrac = 181/360 = 0.50278
    // price = 100 * (1 - 0.05 * 0.50278) = 97.4861
    let args = [
        Value::Number(44927.0), // 2023-01-01
        Value::Number(45108.0), // 2023-07-01
        Value::Number(0.05),
        Value::Number(100.0),
        Value::Number(2.0), // basis: actual/360
    ];
    assert!(approx(pricedisc_fn(&args), 97.4861, 1e-4));
}

#[test]
fn pricedisc_default_basis() {
    // Without optional basis argument (defaults to basis=0: 30/360)
    // Days 30/360: 2023-01-01 to 2023-07-01 = (0)*360 + (6)*30 + 0 = 180
    // yearfrac = 180/360 = 0.5
    // price = 100 * (1 - 0.05 * 0.5) = 97.5
    let args = [
        Value::Number(44927.0), // 2023-01-01
        Value::Number(45108.0), // 2023-07-01
        Value::Number(0.05),
        Value::Number(100.0),
    ];
    assert!(approx(pricedisc_fn(&args), 97.5, 1e-4));
}

// ---------------------------------------------------------------------------
// DISC
// ---------------------------------------------------------------------------

#[test]
fn disc_recovers_known_rate() {
    // DISC is the inverse of PRICEDISC:
    // If PRICEDISC(settlement, maturity, 0.05, 100, 2) = 97.4861,
    // then DISC(settlement, maturity, 97.4861, 100, 2) should return ~0.05
    let args = [
        Value::Number(44927.0), // 2023-01-01
        Value::Number(45108.0), // 2023-07-01
        Value::Number(97.4861111111111),
        Value::Number(100.0),
        Value::Number(2.0),
    ];
    assert!(approx(disc_fn(&args), 0.05, 1e-4));
}

#[test]
fn disc_basic_30_360() {
    // basis=0 (30/360): yearfrac = 180/360 = 0.5
    // price = 100*(1 - 0.05*0.5) = 97.5
    // DISC(settlement, maturity, 97.5, 100) = (100-97.5)/100/0.5 = 0.05
    let args = [
        Value::Number(44927.0), // 2023-01-01
        Value::Number(45108.0), // 2023-07-01
        Value::Number(97.5),
        Value::Number(100.0),
    ];
    assert!(approx(disc_fn(&args), 0.05, 1e-4));
}

// ---------------------------------------------------------------------------
// TBILLPRICE
// ---------------------------------------------------------------------------

#[test]
fn tbillprice_5pct_6month() {
    // TBILLPRICE(settlement=2023-01-01, maturity=2023-07-01, discount=5%)
    // DSM = 181 actual days
    // price = 100 * (1 - 0.05 * 181/360) = 97.4861
    let args = [
        Value::Number(44927.0), // 2023-01-01
        Value::Number(45108.0), // 2023-07-01
        Value::Number(0.05),
    ];
    assert!(approx(tbillprice_fn(&args), 97.4861, 1e-4));
}

// ---------------------------------------------------------------------------
// COUPDAYS
// ---------------------------------------------------------------------------

#[test]
fn coupdays_semiannual_actual() {
    // COUPDAYS(settlement=2023-01-01, maturity=2025-01-01, frequency=2, basis=1)
    // basis=1: actual days — prev coupon 2023-01-01, next coupon 2023-07-01
    // actual days = 181
    let args = [
        Value::Number(44927.0), // 2023-01-01
        Value::Number(45658.0), // 2025-01-01
        Value::Number(2.0),     // semi-annual
        Value::Number(1.0),     // basis: actual/actual
    ];
    assert!(approx(coupdays_fn(&args), 181.0, 1e-4));
}

#[test]
fn coupdays_semiannual_30_360() {
    // COUPDAYS(settlement=2023-01-01, maturity=2025-01-01, frequency=2, basis=0)
    // basis=0: 30/360 → 360/2 = 180
    let args = [
        Value::Number(44927.0), // 2023-01-01
        Value::Number(45658.0), // 2025-01-01
        Value::Number(2.0),     // semi-annual
        Value::Number(0.0),     // basis: 30/360
    ];
    assert!(approx(coupdays_fn(&args), 180.0, 1e-4));
}

// ---------------------------------------------------------------------------
// PRICE happy path
// ---------------------------------------------------------------------------

#[test]
fn price_rate_equals_yield_returns_near_par() {
    // When rate == yield, PRICE should be near redemption (100)
    // settlement=45292 (2024-01-01), maturity=46022 (2025-12-31)
    let args = [
        Value::Number(45292.0),
        Value::Number(46022.0),
        Value::Number(0.05),
        Value::Number(0.05),
        Value::Number(100.0),
        Value::Number(2.0),
    ];
    let result = price_fn(&args);
    if let Value::Number(p) = result {
        assert!((p - 100.0).abs() < 0.1, "price {} should be near 100", p);
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

// ---------------------------------------------------------------------------
// validate_basis
// ---------------------------------------------------------------------------

#[test]
fn basis_0_is_valid() {
    assert_eq!(validate_basis(0.0), Ok(0));
}

#[test]
fn basis_4_is_valid() {
    assert_eq!(validate_basis(4.0), Ok(4));
}

// ---------------------------------------------------------------------------
// validate_frequency
// ---------------------------------------------------------------------------

#[test]
fn frequency_1_annual_valid() {
    assert_eq!(validate_frequency(1.0), Ok(1));
}

#[test]
fn frequency_2_semiannual_valid() {
    assert_eq!(validate_frequency(2.0), Ok(2));
}

#[test]
fn frequency_4_quarterly_valid() {
    assert_eq!(validate_frequency(4.0), Ok(4));
}
