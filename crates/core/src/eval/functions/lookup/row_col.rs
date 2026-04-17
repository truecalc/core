use crate::eval::evaluate_expr;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};
use super::cell_ref::{parse_cell_ref, parse_range_ref};

/// `ROW([cell_ref])` — returns the row number of a cell reference.
/// Without argument, returns 1 (no row context in standalone evaluator).
pub fn row_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 0, 1) {
        return err;
    }
    if args.is_empty() {
        return Value::Number(1.0);
    }
    match &args[0] {
        Expr::Variable(name, _) => {
            // Try range first (A1:D4 → top row of range)
            if let Some((_sc, sr, _ec, _er)) = parse_range_ref(name) {
                return Value::Number(sr as f64);
            }
            // Try single cell ref
            if let Some((_col, row)) = parse_cell_ref(name) {
                return Value::Number(row as f64);
            }
            // Not a cell ref variable: evaluate and forward error or return #N/A
            let v = evaluate_expr(&args[0], ctx);
            match v {
                Value::Error(e) => Value::Error(e),
                _ => Value::Error(ErrorKind::NA),
            }
        }
        _ => {
            // Non-variable argument (e.g. string literal, number): #N/A
            let v = evaluate_expr(&args[0], ctx);
            match v {
                Value::Error(e) => Value::Error(e),
                _ => Value::Error(ErrorKind::NA),
            }
        }
    }
}

/// `COLUMN([cell_ref])` — returns the column number of a cell reference.
/// Without argument, returns 1.
pub fn column_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 0, 1) {
        return err;
    }
    if args.is_empty() {
        return Value::Number(1.0);
    }
    match &args[0] {
        Expr::Variable(name, _) => {
            // Try range first (A1:D4 → left column of range)
            if let Some((sc, _sr, _ec, _er)) = parse_range_ref(name) {
                return Value::Number(sc as f64);
            }
            // Try single cell ref
            if let Some((col, _row)) = parse_cell_ref(name) {
                return Value::Number(col as f64);
            }
            let v = evaluate_expr(&args[0], ctx);
            match v {
                Value::Error(e) => Value::Error(e),
                _ => Value::Error(ErrorKind::NA),
            }
        }
        _ => {
            // Non-variable argument (e.g. string literal, number): #N/A
            let v = evaluate_expr(&args[0], ctx);
            match v {
                Value::Error(e) => Value::Error(e),
                _ => Value::Error(ErrorKind::NA),
            }
        }
    }
}

/// `ROWS(array_or_range)` — returns the number of rows.
/// For a range ref A1:C5 → 5 rows.
/// For an evaluated 2D array → outer count.
/// For a 1D array or scalar → 1.
pub fn rows_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, 1) {
        return err;
    }
    match &args[0] {
        Expr::Variable(name, _) => {
            if let Some((_sc, sr, _ec, er)) = parse_range_ref(name) {
                let rows = if er >= sr { er - sr + 1 } else { sr - er + 1 };
                return Value::Number(rows as f64);
            }
            if parse_cell_ref(name).is_some() {
                return Value::Number(1.0);
            }
            // Fall through to evaluate
            let v = evaluate_expr(&args[0], ctx);
            count_rows_from_value(&v)
        }
        _ => {
            let v = evaluate_expr(&args[0], ctx);
            count_rows_from_value(&v)
        }
    }
}

/// `COLUMNS(array_or_range)` — returns the number of columns.
/// For a range ref A1:D1 → 4 columns.
pub fn columns_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, 1) {
        return err;
    }
    match &args[0] {
        Expr::Variable(name, _) => {
            if let Some((sc, _sr, ec, _er)) = parse_range_ref(name) {
                let cols = if ec >= sc { ec - sc + 1 } else { sc - ec + 1 };
                return Value::Number(cols as f64);
            }
            if parse_cell_ref(name).is_some() {
                return Value::Number(1.0);
            }
            let v = evaluate_expr(&args[0], ctx);
            count_cols_from_value(&v)
        }
        _ => {
            let v = evaluate_expr(&args[0], ctx);
            count_cols_from_value(&v)
        }
    }
}

fn count_rows_from_value(v: &Value) -> Value {
    match v {
        Value::Array(outer) => {
            let is_2d = outer.iter().any(|e| matches!(e, Value::Array(_)));
            if is_2d {
                Value::Number(outer.len() as f64)
            } else {
                Value::Number(1.0)
            }
        }
        Value::Error(e) => Value::Error(e.clone()),
        _ => Value::Number(1.0),
    }
}

fn count_cols_from_value(v: &Value) -> Value {
    match v {
        Value::Array(outer) => {
            let is_2d = outer.iter().any(|e| matches!(e, Value::Array(_)));
            if is_2d {
                // Number of columns in first row
                match outer.first() {
                    Some(Value::Array(row)) => Value::Number(row.len() as f64),
                    _ => Value::Number(1.0),
                }
            } else {
                Value::Number(outer.len() as f64)
            }
        }
        Value::Error(e) => Value::Error(e.clone()),
        _ => Value::Number(1.0),
    }
}
