use crate::eval::{evaluate_expr, functions::{check_arity_len, EvalCtx}};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// `SHEETS([reference])` — returns the number of sheets in a reference or the workbook.
/// In the standalone evaluator there is always exactly 1 sheet.
/// With no argument, returns 1.
/// With a cell-reference argument, returns 1.
/// With a non-reference argument (number, text, etc.) returns `#N/A`.
pub fn sheets_fn(args: &[Expr], _ctx: &mut EvalCtx<'_>) -> Value {
    match args.len() {
        0 => Value::Number(1.0),
        1 => {
            if matches!(args[0], Expr::Variable(_, _)) {
                Value::Number(1.0)
            } else {
                Value::Error(ErrorKind::NA)
            }
        }
        _ => Value::Error(ErrorKind::NA),
    }
}

/// `ERROR.TYPE(error_value)` — returns a number identifying the error type,
/// or `#N/A` if the argument is not an error.
pub fn error_type_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    match val {
        Value::Error(ErrorKind::Null)     => Value::Number(1.0),
        Value::Error(ErrorKind::DivByZero) => Value::Number(2.0),
        Value::Error(ErrorKind::Value)    => Value::Number(3.0),
        Value::Error(ErrorKind::Ref)      => Value::Number(4.0),
        Value::Error(ErrorKind::Name)     => Value::Number(5.0),
        Value::Error(ErrorKind::Num)      => Value::Number(6.0),
        Value::Error(ErrorKind::NA)       => Value::Number(7.0),
        _                                 => Value::Error(ErrorKind::NA),
    }
}

/// `N(value)` — converts a value to a number.
/// Text and Empty return 0. Errors propagate.
pub fn n_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    match val {
        Value::Number(n) | Value::Date(n) => Value::Number(n),
        Value::Bool(b)          => Value::Number(if b { 1.0 } else { 0.0 }),
        Value::Empty | Value::Text(_) | Value::Array(_) => Value::Number(0.0),
        Value::Error(_)         => val,
    }
}

/// `TYPE(value)` — returns a numeric code for the value's type.
/// Does NOT propagate errors; errors return 16.
pub fn type_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, 1) {
        return err;
    }
    let val = evaluate_expr(&args[0], ctx);
    let code = match val {
        Value::Number(_) | Value::Date(_) => 1.0,
        Value::Text(_)   => 2.0,
        Value::Bool(_)   => 4.0,
        Value::Error(_)  => 16.0,
        Value::Array(_)  => 64.0,
        Value::Empty     => 1.0, // Excel treats empty as number
    };
    Value::Number(code)
}

#[cfg(test)]
mod tests;
