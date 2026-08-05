use crate::eval::coercion::to_number;
use crate::eval::evaluate_expr;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// `RAND()` — returns a random number in [0, 1).
/// Uses a per-cell PRF key when available (workbook recalc context);
/// falls back to SystemTime for bare Engine::evaluate calls.
pub fn rand_lazy(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 0, 0) {
        return err;
    }
    Value::Number(ctx.ctx.draw_rand(0))
}

/// `RANDBETWEEN(low, high)` — returns a random integer in [low, high] inclusive.
pub fn randbetween_lazy(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 2, 2) {
        return err;
    }
    let low_val = evaluate_expr(&args[0], ctx);
    let high_val = evaluate_expr(&args[1], ctx);
    let low = match to_number(low_val) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let high = match to_number(high_val) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let lo = low.ceil() as i64;
    let hi = high.floor() as i64;
    if lo > hi {
        return Value::Error(ErrorKind::Num);
    }
    let raw = ctx.ctx.draw_rand(0);
    let range = (hi - lo + 1) as f64;
    Value::Number(lo as f64 + (raw * range).floor())
}

#[cfg(test)]
mod tests;
