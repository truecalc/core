use crate::eval::evaluate_expr;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// `CHOOSE(index, value1, value2, ...)` — returns the value at the 1-based index.
/// Lazy: evaluates only the selected argument.
pub fn choose_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 2, 255) {
        return err;
    }

    let index_val = evaluate_expr(&args[0], ctx);
    let index = match index_val {
        Value::Number(n) => {
            let n = n.trunc() as i64;
            if n < 1 { return Value::Error(ErrorKind::Num); }
            n as usize
        }
        // Bool coercion: TRUE→1, FALSE→0 (out of range → #NUM!)
        Value::Bool(b) => {
            if b { 1usize } else { return Value::Error(ErrorKind::Num); }
        }
        // Empty (omitted arg) → treated as 0 → out of range → #NUM!
        Value::Empty => return Value::Error(ErrorKind::Num),
        Value::Error(e) => return Value::Error(e),
        Value::ErrorMsg(e, m) => return Value::ErrorMsg(e, m),
        _ => return Value::Error(ErrorKind::Value),
    };

    let choice_idx = index; // 1-based index into args[1..]
    if choice_idx >= args.len() {
        return Value::Error(ErrorKind::Num);
    }

    evaluate_expr(&args[choice_idx], ctx)
}
