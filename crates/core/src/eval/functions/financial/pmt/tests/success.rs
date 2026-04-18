use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn monthly_loan_payment() {
    // PMT(0.05/12, 60, 10000) ≈ -188.71
    let rate = Value::Number(0.05 / 12.0);
    let nper = Value::Number(60.0);
    let pv   = Value::Number(10000.0);
    assert!(approx(pmt_fn(&[rate, nper, pv]), -188.71, 0.01));
}

#[test]
fn zero_rate_loan() {
    // PMT(0, 10, 1000) = -100
    let args = [Value::Number(0.0), Value::Number(10.0), Value::Number(1000.0)];
    assert_eq!(pmt_fn(&args), Value::Number(-100.0));
}

#[test]
fn with_future_value() {
    // PMT(0.1, 5, 1000, 500) — should be negative payment
    let args = [
        Value::Number(0.1),
        Value::Number(5.0),
        Value::Number(1000.0),
        Value::Number(500.0),
    ];
    let result = pmt_fn(&args);
    assert!(matches!(result, Value::Number(n) if n < 0.0));
}

#[test]
fn beginning_of_period_type1() {
    // type=1 (annuity-due): payment at beginning of period is smaller in abs value
    // PMT(0.1, 5, 1000, 0, 0) ≈ -263.80, PMT(0.1, 5, 1000, 0, 1) ≈ -239.82
    let args_end   = [Value::Number(0.1), Value::Number(5.0), Value::Number(1000.0), Value::Number(0.0), Value::Number(0.0)];
    let args_begin = [Value::Number(0.1), Value::Number(5.0), Value::Number(1000.0), Value::Number(0.0), Value::Number(1.0)];
    let end   = pmt_fn(&args_end);
    let begin = pmt_fn(&args_begin);
    // beginning-of-period payment has a smaller absolute value (one fewer compounding period)
    if let (Value::Number(e), Value::Number(b)) = (end, begin) {
        assert!(approx(Value::Number(e), -263.80, 0.01), "type0 expected ≈ -263.80, got {e}");
        assert!(approx(Value::Number(b), -239.82, 0.01), "type1 expected ≈ -239.82, got {b}");
        assert!(b.abs() < e.abs(), "type=1 abs value should be smaller than type=0");
    } else {
        panic!("expected numbers");
    }
}
