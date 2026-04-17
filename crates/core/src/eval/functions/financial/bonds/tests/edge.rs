use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn pricedisc_zero_discount_returns_redemption() {
    // If discount = 0, price = redemption regardless of dates
    let args = [
        Value::Number(44927.0), // 2023-01-01
        Value::Number(45108.0), // 2023-07-01
        Value::Number(0.0),     // discount = 0%
        Value::Number(100.0),
    ];
    assert!(approx(pricedisc_fn(&args), 100.0, 1e-9));
}

#[test]
fn coupdays_annual_frequency() {
    // COUPDAYS with frequency=1 (annual): 30/360 basis → 360 days
    let args = [
        Value::Number(44927.0), // 2023-01-01
        Value::Number(45658.0), // 2025-01-01
        Value::Number(1.0),     // annual
        Value::Number(0.0),     // basis: 30/360
    ];
    assert!(approx(coupdays_fn(&args), 360.0, 1e-4));
}
