use crate::eval::evaluate_expr;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// `FORMULATEXT(reference)` — returns the formula string of a cell reference.
/// In a standalone evaluator, all literals have no formula → #N/A.
pub fn formulatext_fn(args: &[Expr], _ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, 1) {
        return err;
    }
    Value::Error(ErrorKind::NA)
}

/// `GETPIVOTDATA(...)` — always returns #N/A in a standalone evaluator.
pub fn getpivotdata_fn(args: &[Expr], _ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    Value::Error(ErrorKind::NA)
}

/// `OFFSET(reference, rows, cols, [height], [width])` — not implementable without cell grid.
pub fn offset_fn(args: &[Expr], _ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    Value::Error(ErrorKind::NA)
}

/// `SHEET([name])` — returns the sheet index.
/// With no argument, returns 1 (standalone evaluator has 1 sheet).
/// With a text argument → #REF! (sheet not found).
pub fn sheet_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    match args.len() {
        0 => Value::Number(1.0),
        1 => {
            let val = evaluate_expr(&args[0], ctx);
            match val {
                Value::Text(_) => Value::Error(ErrorKind::Ref),
                Value::Error(e) => Value::Error(e),
                _ => Value::Error(ErrorKind::NA),
            }
        }
        _ => Value::Error(ErrorKind::NA),
    }
}
