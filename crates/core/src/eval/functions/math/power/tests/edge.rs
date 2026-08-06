use super::super::*;
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    crate::Engine::sheets().evaluate(formula, &HashMap::new())
}

#[test]
fn power_negative_base_integer_exp() {
    assert_eq!(
        power_fn(&[Value::Number(-2.0), Value::Number(3.0)]),
        Value::Number(-8.0)
    );
}

#[test]
fn power_fractional_exponent() {
    // 4^0.5 = 2
    assert_eq!(
        power_fn(&[Value::Number(4.0), Value::Number(0.5)]),
        Value::Number(2.0)
    );
}

#[test]
fn power_overflow_returns_num_error() {
    assert_eq!(
        power_fn(&[Value::Number(f64::MAX), Value::Number(2.0)]),
        Value::Error(ErrorKind::Num)
    );
}

// ── POWER(x,y) vs x^y parity (core#846) ─────────────────────────────────────
//
// POWER() and the `^` operator are documented to be equivalent and must
// return the same result for identical inputs. Google Sheets conformance
// fixtures (fixtures/google_sheets/math.tsv) record POWER(-8,1/3) = -2 and
// POWER(-27,1/3) = -3 as ground truth (odd-root real result for a negative
// base), so the `^` operator -- not POWER() -- was the one out of step.

#[test]
fn power_and_caret_agree_on_negative_base_cube_root() {
    assert_eq!(run("=POWER(-8,1/3)"), run("=(-8)^(1/3)"));
    assert_eq!(run("=POWER(-8,1/3)"), Value::Number(-2.0));
}

#[test]
fn power_and_caret_agree_on_negative_27_cube_root() {
    assert_eq!(run("=POWER(-27,1/3)"), run("=(-27)^(1/3)"));
    assert_eq!(run("=POWER(-27,1/3)"), Value::Number(-3.0));
}

#[test]
fn power_and_caret_agree_on_negative_base_square_root() {
    // Even root of a negative base has no real result in either form.
    assert_eq!(run("=POWER(-8,0.5)"), run("=(-8)^(0.5)"));
    assert_eq!(run("=POWER(-8,0.5)"), Value::Error(ErrorKind::Num));
}

#[test]
fn power_and_caret_agree_on_negative_base_fifth_root() {
    assert_eq!(run("=POWER(-32,1/5)"), run("=(-32)^(1/5)"));
    assert_eq!(run("=POWER(-32,1/5)"), Value::Number(-2.0));
}
