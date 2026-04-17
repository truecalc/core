use crate::eval::evaluate_expr;
use crate::eval::functions::EvalCtx;
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// `LET(name1, val1, ..., nameN, valN, body)` — bind named values and evaluate body.
///
/// Rules:
/// - Must have an odd number of arguments ≥ 3 (at least one binding + body).
/// - Name arguments must be identifier expressions.
/// - Bindings are evaluated in order; later bindings can reference earlier ones.
/// - Bindings are scoped to the body; context is restored after evaluation.
pub fn let_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    let count = args.len();
    // Need at least 3 args and an odd count (pairs of name/value + 1 body)
    if count < 3 || count.is_multiple_of(2) {
        return Value::Error(ErrorKind::NA);
    }

    let pair_count = (count - 1) / 2;
    let body = &args[count - 1];

    // Validate that all name slots are identifiers before binding anything.
    for i in 0..pair_count {
        match &args[i * 2] {
            Expr::Variable(_, _) => {}
            _ => return Value::Error(ErrorKind::Value),
        }
    }

    // Bind each name to its value in order (later bindings see earlier ones).
    let mut saved: Vec<(String, Option<Value>)> = Vec::with_capacity(pair_count);
    for i in 0..pair_count {
        let name = match &args[i * 2] {
            Expr::Variable(n, _) => n.to_uppercase(),
            _ => unreachable!(), // validated above
        };
        let val = evaluate_expr(&args[i * 2 + 1], ctx);
        let old = ctx.ctx.set(name.clone(), val);
        saved.push((name, old));
    }

    let result = evaluate_expr(body, ctx);

    // Restore context in reverse order.
    for (name, old) in saved.into_iter().rev() {
        match old {
            Some(v) => { ctx.ctx.set(name, v); }
            None    => { ctx.ctx.remove(&name); }
        }
    }

    result
}

#[cfg(test)]
mod tests;
