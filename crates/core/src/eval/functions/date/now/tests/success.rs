use super::super::*;
use crate::eval::functions::{EvalCtx, Registry};
use crate::eval::Context;
use crate::types::Value;

fn call(now_serial: Option<f64>) -> Value {
    let registry = Registry::new();
    let mut ctx = Context::empty();
    ctx.now_serial = now_serial;
    let mut eval_ctx = EvalCtx::new(ctx, &registry);
    now_fn(&[], &mut eval_ctx)
}

/// Serial for 2024-01-01 = 45292.
const MIN_SERIAL: f64 = 45292.0;

#[test]
fn returns_a_number() {
    assert!(matches!(call(None), Value::Number(_)));
}

#[test]
fn result_is_after_2024_jan_01() {
    if let Value::Number(n) = call(None) {
        assert!(n >= MIN_SERIAL, "now serial {n} should be >= {MIN_SERIAL}");
    } else {
        panic!("expected Number");
    }
}

#[test]
fn fractional_part_is_between_0_and_1() {
    if let Value::Number(n) = call(None) {
        let frac = n.fract();
        assert!(frac >= 0.0 && frac < 1.0, "fractional part {frac} out of range");
    } else {
        panic!("expected Number");
    }
}

#[test]
fn pinned_context_returns_now_serial_verbatim() {
    assert_eq!(call(Some(46180.5)), Value::Number(46180.5));
}
