use crate::eval::coercion::to_bool;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::eval::evaluate_expr;
use crate::parser::ast::Expr;
use crate::types::Value;

/// `XOR(logical1, ...)` — TRUE if an odd number of arguments evaluate to TRUE.
///
/// Returns `#VALUE!` with no args or if any arg cannot be coerced to bool.
/// Propagates errors from arguments immediately.
pub fn xor_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, usize::MAX) {
        return err;
    }
    let mut true_count = 0usize;
    for arg in args {
        let val = evaluate_expr(arg, ctx);
        match to_bool(val) {
            Ok(true) => true_count += 1,
            Ok(false) => {}
            Err(e) => return e,
        }
    }
    Value::Bool(true_count % 2 == 1)
}
