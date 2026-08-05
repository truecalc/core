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

#[test]
fn result_is_reasonable_upper_bound() {
    // Sanity: the result should not be unreasonably large (e.g., > serial for 2100-01-01 ≈ 73051).
    if let Value::Date(n) = call(None) {
        assert!(n < 73051.0, "today serial {n} seems unreasonably large");
    } else {
        panic!("expected Date");
    }
}
