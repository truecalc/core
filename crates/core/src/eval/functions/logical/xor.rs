use crate::eval::coercion::to_bool;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::eval::evaluate_expr;
use crate::parser::ast::Expr;
use crate::types::Value;

/// `XOR(logical1, ...)` — TRUE if an odd number of arguments evaluate to TRUE.
///
/// Returns `#VALUE!` with no args or if any arg cannot be coerced to bool.
/// Propagates errors from arguments immediately. Array arguments are flattened:
/// each element is counted individually (GS/Excel compatible).
pub fn xor_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, usize::MAX) {
        return err;
    }
    let mut true_count = 0usize;
    for arg in args {
        let val = evaluate_expr(arg, ctx);
        match count_trues(val) {
            Ok(n) => true_count += n,
            Err(e) => return e,
        }
    }
    Value::Bool(true_count % 2 == 1)
}

/// Recursively count the number of truthy elements in a value (array or scalar).
fn count_trues(val: Value) -> Result<usize, Value> {
    match val {
        Value::Array(elems) => {
            let mut count = 0usize;
            for elem in elems {
                count += count_trues(elem)?;
            }
            Ok(count)
        }
        other => match to_bool(other) {
            Ok(true) => Ok(1),
            Ok(false) => Ok(0),
            Err(e) => Err(e),
        },
    }
}
