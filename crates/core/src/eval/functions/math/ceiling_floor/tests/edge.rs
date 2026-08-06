use super::super::*;
use crate::types::Value;

#[test]
fn ceiling_negative_both_negative_sig() {
    // CEILING(-4.5, -1) → -5
    assert_eq!(
        ceiling_fn(&[Value::Number(-4.5), Value::Number(-1.0)]),
        Value::Number(-5.0)
    );
}

#[test]
fn floor_negative_both_negative_sig() {
    // FLOOR(-4.5, -1) → -4
    assert_eq!(
        floor_fn(&[Value::Number(-4.5), Value::Number(-1.0)]),
        Value::Number(-4.0)
    );
}

#[test]
fn ceiling_significance_zero_returns_zero() {
    assert_eq!(
        ceiling_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn floor_significance_zero_nonzero_returns_div_by_zero() {
    // GS: FLOOR(x, 0) with x ≠ 0 → #DIV/0!
    use crate::types::ErrorKind;
    assert_eq!(
        floor_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

// Regression coverage for issue #845.
//
// The issue reported FLOOR(x, 0) returning #DIV/0! as a bug, expecting 0 to
// match CEILING(x, 0) and FLOOR.MATH(x, 0). Investigation against the
// immutable Google Sheets conformance fixtures
// (crates/core/tests/fixtures/google_sheets/math.tsv) shows this is real
// Google Sheets behavior, not an engine defect:
//   - `=FLOOR(5,0)` (row "factor of zero produces error")            -> #DIV/0!
//   - `=IFERROR(FLOOR(5,0),"fallback")`                                -> "fallback"
//   - `=CEILING(3,0)` (row "factor of zero causes error")            -> 0
//   - `=FLOOR.MATH(5,0)` (row "significance zero...")                -> 0
// Google Sheets itself is inconsistent between FLOOR and CEILING/FLOOR.MATH
// for zero significance; TrueCalc correctly mirrors that ground truth, so
// FLOOR(x, 0) must keep returning #DIV/0! for every nonzero x (positive or
// negative), while FLOOR(0, 0) stays 0. These tests lock that in across the
// positive/negative/zero x space named in the issue's acceptance criteria.
#[test]
fn floor_significance_zero_positive_x_returns_div_by_zero() {
    use crate::types::ErrorKind;
    assert_eq!(
        floor_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn floor_significance_zero_negative_x_returns_div_by_zero() {
    use crate::types::ErrorKind;
    assert_eq!(
        floor_fn(&[Value::Number(-5.0), Value::Number(0.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
    assert_eq!(
        floor_fn(&[Value::Number(-3.0), Value::Number(0.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn floor_significance_zero_zero_x_returns_zero() {
    assert_eq!(
        floor_fn(&[Value::Number(0.0), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn floor_significance_float_zero_matches_integer_zero() {
    // FLOOR(5, 0.0) must behave identically to FLOOR(5, 0).
    use crate::types::ErrorKind;
    assert_eq!(
        floor_fn(&[Value::Number(5.0), Value::Number(0.0_f64)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn floor_significance_zero_diverges_from_ceiling_and_floor_math_by_design() {
    // Documents the real Google Sheets asymmetry named in issue #845: FLOOR
    // errors on zero significance for nonzero x, while CEILING and
    // FLOOR.MATH both return 0. This is expected, not a bug.
    use crate::eval::functions::math::ceiling_floor_math::floor_math_fn;
    use crate::types::ErrorKind;

    assert_eq!(
        floor_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
    assert_eq!(
        ceiling_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Number(0.0)
    );
    assert_eq!(
        floor_math_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}
