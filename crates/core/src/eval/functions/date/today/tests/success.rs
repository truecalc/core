use super::super::*;
use crate::eval::functions::{EvalCtx, Registry};
use crate::eval::Context;
use crate::types::Value;

fn call(now_serial: Option<f64>) -> Value {
    let registry = Registry::new();
    let mut ctx = Context::empty();
    ctx.now_serial = now_serial;
    let mut eval_ctx = EvalCtx::new(ctx, &registry);
    today_fn(&[], &mut eval_ctx)
}

/// Serial for 2024-01-01 = 45292. Ambient TODAY() must be at or above that.
const MIN_SERIAL: f64 = 45292.0;

#[test]
fn returns_a_date() {
    assert!(matches!(call(None), Value::Date(_)));
}

#[test]
fn result_is_after_2024_jan_01() {
    if let Value::Date(n) = call(None) {
        assert!(n >= MIN_SERIAL, "today serial {n} should be >= {MIN_SERIAL}");
    } else {
        panic!("expected Date");
    }
}

#[test]
fn result_has_no_fractional_part() {
    if let Value::Date(n) = call(None) {
        assert_eq!(n.fract(), 0.0, "TODAY() should be a whole number, got {n}");
    } else {
        panic!("expected Date");
    }
}

#[test]
fn pinned_context_returns_floor_of_now_serial() {
    // workbook.tsv volatile row: =TODAY() → 46180 (date) under
    // meta.evaluatedAt 2026-06-07T23:50:56.808Z, Etc/GMT (PR #559).
    let evaluated_at_serial = 46180.0 + (23.0 * 3600.0 + 50.0 * 60.0 + 56.0) / 86400.0;
    assert_eq!(call(Some(evaluated_at_serial)), Value::Date(46180.0));
}

#[test]
fn pinned_context_is_deterministic() {
    assert_eq!(call(Some(46180.25)), call(Some(46180.25)));
}
