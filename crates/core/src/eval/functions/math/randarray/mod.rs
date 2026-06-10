use crate::eval::coercion::to_number;
use crate::eval::evaluate_expr;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// `RANDARRAY([rows], [cols], [min], [max], [integer])`
///
/// Returns an array of random numbers.
/// With no args: returns a single random number (equivalent to RAND()).
/// With rows/cols: returns a nested 2D array (rows × cols).
/// Uses a per-cell PRF key when available; falls back to SystemTime.
pub fn randarray_lazy(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Number(ctx.ctx.draw_rand(0));
    }
    if let Some(err) = check_arity_len(args.len(), 1, 5) {
        return err;
    }
    let rows_val = evaluate_expr(&args[0], ctx);
    let rows = match to_number(rows_val) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if rows <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let rows = rows as usize;
    let cols = if args.len() >= 2 {
        let cv = evaluate_expr(&args[1], ctx);
        match to_number(cv) {
            Err(e) => return e,
            Ok(v) => {
                if v <= 0.0 {
                    return Value::Error(ErrorKind::Num);
                }
                v as usize
            }
        }
    } else {
        1
    };
    let total = rows * cols;
    let nums = ctx.ctx.draw_rand_n(total);
    let outer: Vec<Value> = (0..rows)
        .map(|r| Value::Array((0..cols).map(|c| Value::Number(nums[r * cols + c])).collect()))
        .collect();
    Value::Array(outer)
}

#[cfg(test)]
mod tests;
