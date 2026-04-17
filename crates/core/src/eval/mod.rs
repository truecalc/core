pub mod context;
pub mod coercion;
pub mod functions;

pub use context::Context;
pub use functions::{EvalCtx, FunctionMeta, Registry};

use crate::parser::ast::{BinaryOp, Expr, UnaryOp};
use crate::types::{ErrorKind, Value};

use coercion::{to_number, to_string_val};
use functions::FunctionKind;

/// Walk an expression tree and produce a [`Value`].
///
/// Variables are resolved from `ctx.ctx`; functions are dispatched through
/// `ctx.registry`. Eager functions receive pre-evaluated arguments; lazy
/// functions (e.g. `IF`) receive raw [`Expr`] nodes and control their own
/// evaluation order.
pub fn evaluate_expr(expr: &Expr, ctx: &mut EvalCtx<'_>) -> Value {
    match expr {
        // ── Leaf nodes ──────────────────────────────────────────────────────
        Expr::Number(n, _) => {
            if n.is_finite() {
                Value::Number(*n)
            } else {
                Value::Error(ErrorKind::Num)
            }
        }
        Expr::Text(s, _)   => Value::Text(s.clone()),
        Expr::Bool(b, _)   => Value::Bool(*b),
        Expr::Variable(name, _) => ctx.ctx.get(name),

        // ── Unary ops ───────────────────────────────────────────────────────
        Expr::UnaryOp { op, operand, .. } => {
            let val = evaluate_expr(operand, ctx);
            match to_number(val) {
                Err(e) => e,
                Ok(n)  => match op {
                    UnaryOp::Neg     => Value::Number(-n),
                    UnaryOp::Percent => Value::Number(n / 100.0),
                },
            }
        }

        // ── Binary ops ──────────────────────────────────────────────────────
        Expr::BinaryOp { op, left, right, .. } => {
            let lv = evaluate_expr(left, ctx);
            let rv = evaluate_expr(right, ctx);
            eval_binary(op, lv, rv)
        }

        // ── Array literals ──────────────────────────────────────────────────
        Expr::Array(elems, _) => {
            let mut values = Vec::with_capacity(elems.len());
            for elem in elems {
                let v = evaluate_expr(elem, ctx);
                values.push(v);
            }
            Value::Array(values)
        }

        // ── Function calls ──────────────────────────────────────────────────
        Expr::FunctionCall { name, args, .. } => {
            match ctx.registry.get(name) {
                None => Value::Error(ErrorKind::Name),
                Some(FunctionKind::Lazy(f)) => {
                    // Copy the fn pointer out to avoid holding a borrow on ctx.registry
                    // while also mutably borrowing ctx itself.
                    let f: functions::LazyFn = *f;
                    f(args, ctx)
                }
                Some(FunctionKind::Eager(f)) => {
                    let f: functions::EagerFn = *f;
                    // Evaluate all args; return first error encountered.
                    let mut evaluated = Vec::with_capacity(args.len());
                    for arg in args {
                        let v = evaluate_expr(arg, ctx);
                        if matches!(v, Value::Error(_)) {
                            return v;
                        }
                        evaluated.push(v);
                    }
                    f(&evaluated)
                }
            }
        }

        // ── Immediately-invoked lambda application ──────────────────────────
        Expr::Apply { func, call_args, .. } => {
            eval_apply(func, call_args, ctx)
        }
    }
}

/// Evaluate an immediately-invoked function application `func(call_args)`.
fn eval_apply(func: &Expr, call_args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    let (lambda_params, body) = match func {
        Expr::FunctionCall { name, args: lambda_args, .. } if name == "LAMBDA" => {
            if lambda_args.is_empty() {
                return Value::Error(ErrorKind::NA);
            }
            let param_count = lambda_args.len() - 1;
            let mut params: Vec<String> = Vec::with_capacity(param_count);
            for param_expr in &lambda_args[..param_count] {
                match param_expr {
                    Expr::Variable(n, _) => params.push(n.to_uppercase()),
                    _ => return Value::Error(ErrorKind::Value),
                }
            }
            let body = &lambda_args[lambda_args.len() - 1];
            (params, body)
        }
        _ => return Value::Error(ErrorKind::Value),
    };

    if call_args.len() != lambda_params.len() {
        return Value::Error(ErrorKind::NA);
    }

    let mut evaluated_args: Vec<Value> = Vec::with_capacity(call_args.len());
    for arg in call_args {
        let v = evaluate_expr(arg, ctx);
        if matches!(v, Value::Error(_)) {
            return v;
        }
        evaluated_args.push(v);
    }

    let mut saved: Vec<(String, Option<Value>)> = Vec::with_capacity(lambda_params.len());
    for (param, val) in lambda_params.iter().zip(evaluated_args) {
        let old = ctx.ctx.vars.insert(param.clone(), val);
        saved.push((param.clone(), old));
    }

    let result = evaluate_expr(body, ctx);

    for (name, old_val) in saved.into_iter().rev() {
        match old_val {
            Some(v) => { ctx.ctx.vars.insert(name, v); }
            None    => { ctx.ctx.vars.remove(&name); }
        }
    }

    result
}

// ── Type ordering for cross-type comparisons (Excel semantics) ───────────────
// Number < Text < Bool  (Empty counts as Number)
fn type_rank(v: &Value) -> u8 {
    match v {
        Value::Number(_) | Value::Date(_) | Value::Empty => 0,
        Value::Text(_)                  => 1,
        Value::Bool(_)                  => 2,
        // Error and Array cannot reach compare_values through the normal eval path
        // (eval_binary guards against errors before calling compare_values).
        Value::Error(_) | Value::Array(_) => 3,
    }
}

fn eval_binary(op: &BinaryOp, lv: Value, rv: Value) -> Value {
    // ── Array broadcasting ───────────────────────────────────────────────────
    match (&lv, &rv) {
        (Value::Array(elems), _) => {
            let result: Vec<Value> = elems
                .iter()
                .map(|e| eval_binary(op, e.clone(), rv.clone()))
                .collect();
            return Value::Array(result);
        }
        (_, Value::Array(elems)) => {
            let result: Vec<Value> = elems
                .iter()
                .map(|e| eval_binary(op, lv.clone(), e.clone()))
                .collect();
            return Value::Array(result);
        }
        _ => {}
    }
    match op {
        // ── Arithmetic ──────────────────────────────────────────────────────
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Pow => {
            let ln = match to_number(lv) { Ok(n) => n, Err(e) => return e };
            let rn = match to_number(rv) { Ok(n) => n, Err(e) => return e };
            let result = match op {
                BinaryOp::Add => ln + rn,
                BinaryOp::Sub => ln - rn,
                BinaryOp::Mul => ln * rn,
                BinaryOp::Div => {
                    if rn == 0.0 {
                        return Value::Error(ErrorKind::DivByZero);
                    }
                    ln / rn
                }
                BinaryOp::Pow => ln.powf(rn),
                // Safety: outer match arm covers exactly Add|Sub|Mul|Div|Pow; Concat and comparison ops are handled separately.
                _ => unreachable!(),
            };
            if !result.is_finite() {
                return Value::Error(ErrorKind::Num);
            }
            Value::Number(result)
        }

        // ── Concatenation ───────────────────────────────────────────────────
        BinaryOp::Concat => {
            let ls = match to_string_val(lv) { Ok(s) => s, Err(e) => return e };
            let rs = match to_string_val(rv) { Ok(s) => s, Err(e) => return e };
            Value::Text(ls + &rs)
        }

        // ── Comparisons ─────────────────────────────────────────────────────
        BinaryOp::Eq | BinaryOp::Ne
        | BinaryOp::Lt | BinaryOp::Gt
        | BinaryOp::Le | BinaryOp::Ge => {
            // Error propagation: left side first.
            if let Value::Error(_) = &lv { return lv; }
            if let Value::Error(_) = &rv { return rv; }

            let result = compare_values(op, &lv, &rv);
            Value::Bool(result)
        }
    }
}

/// Compare two (non-error) values with Excel ordering semantics.
fn compare_values(op: &BinaryOp, lv: &Value, rv: &Value) -> bool {
    match (lv, rv) {
        (Value::Number(a), Value::Number(b)) => apply_cmp(op, a.partial_cmp(b)),
        (Value::Date(a),   Value::Date(b))   => apply_cmp(op, a.partial_cmp(b)),
        (Value::Date(a),   Value::Number(b)) => apply_cmp(op, a.partial_cmp(b)),
        (Value::Number(a), Value::Date(b))   => apply_cmp(op, a.partial_cmp(b)),
        (Value::Text(a),   Value::Text(b))   => apply_cmp(op, Some(a.cmp(b))),
        (Value::Bool(a),   Value::Bool(b))   => apply_cmp(op, Some(a.cmp(b))),
        (Value::Empty,     Value::Empty)     => apply_cmp(op, Some(std::cmp::Ordering::Equal)),
        // Empty acts as Number(0)
        (Value::Empty, Value::Number(b))     => apply_cmp(op, 0.0f64.partial_cmp(b)),
        (Value::Number(a), Value::Empty)     => apply_cmp(op, a.partial_cmp(&0.0f64)),
        // Cross-type: use type rank
        _ => {
            let lr = type_rank(lv);
            let rr = type_rank(rv);
            match op {
                BinaryOp::Eq => false,
                BinaryOp::Ne => true,
                BinaryOp::Lt => lr < rr,
                BinaryOp::Gt => lr > rr,
                BinaryOp::Le => lr <= rr,
                BinaryOp::Ge => lr >= rr,
                // Safety: outer match arm covers exactly Eq|Ne|Lt|Gt|Le|Ge; arithmetic and Concat ops are handled separately.
                _ => unreachable!(),
            }
        }
    }
}

fn apply_cmp(op: &BinaryOp, ord: Option<std::cmp::Ordering>) -> bool {
    match ord {
        // NaN: per Value::Number invariant this should not occur after the is_finite() guard;
        // returning false matches Excel semantics if it somehow does.
        None => false,
        Some(o) => match op {
            BinaryOp::Eq => o.is_eq(),
            BinaryOp::Ne => o.is_ne(),
            BinaryOp::Lt => o.is_lt(),
            BinaryOp::Gt => o.is_gt(),
            BinaryOp::Le => o.is_le(),
            BinaryOp::Ge => o.is_ge(),
            // Safety: apply_cmp is only called from compare_values which is only called from eval_binary's comparison arm (Eq|Ne|Lt|Gt|Le|Ge).
            _ => unreachable!(),
        },
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests;
