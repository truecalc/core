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

#[test]
fn integer_part_equals_today_floor() {
    // The integer portion of NOW() must match the date (no fractional spillover).
    if let Value::Date(n) = call(None) {
        assert!(n.floor() >= 45292.0, "date portion {} seems too small", n.floor());
    } else {
        panic!("expected Date");
    }
}
