use crate::eval::functions::{check_arity, check_arity_len, EvalCtx};
use crate::eval::evaluate_expr;
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

// ── Eager versions (used by unit tests) ──────────────────────────────────────

pub fn isnumber_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    Value::Bool(matches!(args[0], Value::Number(_)))
}

pub fn istext_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    Value::Bool(matches!(args[0], Value::Text(_)))
}

pub fn iserror_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    Value::Bool(matches!(args[0], Value::Error(_)))
}

pub fn isblank_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    Value::Bool(matches!(args[0], Value::Empty))
}

pub fn isna_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    Value::Bool(matches!(args[0], Value::Error(ErrorKind::NA)))
}

// ── Lazy versions (registered — can inspect error arguments) ─────────────────

/// `ISNUMBER(value)` — TRUE if value is a Number.
pub fn isnumber_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    Value::Bool(matches!(val, Value::Number(_)))
}

/// `ISTEXT(value)` — TRUE if value is a Text string.
pub fn istext_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    Value::Bool(matches!(val, Value::Text(_)))
}

/// `ISERROR(value)` — TRUE if value is any Error.
/// Must be lazy so the evaluator does not short-circuit on error arguments.
pub fn iserror_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    Value::Bool(matches!(val, Value::Error(_)))
}

/// `ISBLANK(value)` — TRUE if value is Empty.
pub fn isblank_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    Value::Bool(matches!(val, Value::Empty))
}

/// `ISNA(value)` — TRUE if value is `Error(NA)` specifically.
pub fn isna_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    Value::Bool(matches!(val, Value::Error(ErrorKind::NA)))
}

/// `ISERR(value)` — TRUE if value is any Error **except** `#N/A`.
pub fn iserr_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    let is_err = match &val {
        Value::Error(e) => *e != ErrorKind::NA,
        _ => false,
    };
    Value::Bool(is_err)
}

/// `ISLOGICAL(value)` — TRUE if value is a boolean (TRUE or FALSE).
pub fn islogical_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    Value::Bool(matches!(val, Value::Bool(_)))
}

/// `ISNONTEXT(value)` — TRUE if value is NOT text (including errors and empty).
pub fn isnontext_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    Value::Bool(!matches!(val, Value::Text(_)))
}

/// `ISREF(value)` — TRUE if the argument is a cell reference (Variable node).
/// Errors in the argument are NOT propagated; they return FALSE.
/// Only 0 args → `#N/A`.
pub fn isref_fn(args: &[Expr], _ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    Value::Bool(matches!(args[0], Expr::Variable(_, _)))
}

/// `ISFORMULA(value)` — TRUE if the argument is a cell ref containing a formula.
/// Requires exactly 1 arg that is a cell reference; otherwise → `#N/A`.
/// We have no formula context, so a valid cell ref returns FALSE.
pub fn isformula_fn(args: &[Expr], _ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    if !matches!(args[0], Expr::Variable(_, _)) {
        return Value::Error(ErrorKind::NA);
    }
    Value::Bool(false)
}

#[cfg(test)]
mod tests;
