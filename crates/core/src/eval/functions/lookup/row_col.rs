use crate::eval::evaluate_expr;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::parser::ast::Expr;
use crate::parser::refs::Ref;
use crate::types::{ErrorKind, Value};
use super::cell_ref::{parse_cell_ref, parse_range_ref};
use super::misc::parse_offset_ref;

// ---------------------------------------------------------------------------
// Helpers: extract ref geometry from a lazy argument
// ---------------------------------------------------------------------------

/// Try to resolve an expression argument to a cell-ref string without evaluating it.
/// Handles:
///   - `INDIRECT("A1")` / `INDIRECT("A1:C3")` — extracts the literal string arg
///   - `OFFSET(INDIRECT("A1"), r, c, [h], [w])` — evaluates to get the offset tag
///
/// Returns the resolved cell/range string, or None if unrecognised.
fn extract_ref_string_from_expr(arg: &Expr, ctx: &mut EvalCtx<'_>) -> Option<String> {
    match arg {
        // INDIRECT("literal") — pull the literal out without evaluating INDIRECT.
        Expr::FunctionCall { name, args, .. } if name.eq_ignore_ascii_case("INDIRECT") => {
            if !args.is_empty() {
                match &args[0] {
                    Expr::Text(s, _) => return Some(s.clone()),
                    _ => {
                        // Evaluate the arg to get the string.
                        let v = evaluate_expr(&args[0], ctx);
                        if let Value::Text(s) = v { return Some(s); }
                    }
                }
            }
            None
        }
        // OFFSET(...) — evaluate it; it returns an __offset__:... tag.
        Expr::FunctionCall { name, .. } if name.eq_ignore_ascii_case("OFFSET") => {
            let v = evaluate_expr(arg, ctx);
            match v {
                Value::Text(s) if s.starts_with("__offset__:") => Some(s),
                Value::Error(_) | Value::ErrorMsg(_, _) => None,
                _ => None,
            }
        }
        _ => None,
    }
}

/// Resolve a lazy arg to (row, col, height, width) — all 1-based.
/// For a plain cell ref: height=1, width=1.
/// Returns None if the arg doesn't resolve to a known ref.
fn resolve_lazy_arg_geometry(arg: &Expr, ctx: &mut EvalCtx<'_>)
    -> Option<(usize, usize, usize, usize)>
{
    // Try to get a ref string from INDIRECT/OFFSET.
    if let Some(s) = extract_ref_string_from_expr(arg, ctx) {
        return decode_ref_string_geometry(&s);
    }
    // Expr::Reference produced by the parser for bare cell/range tokens.
    match arg {
        Expr::Reference(r, _) => match r {
            Ref::Cell { addr, .. } => {
                return Some((addr.row as usize, addr.col as usize, 1, 1));
            }
            Ref::Range { start, end, .. } => {
                let h = if end.row >= start.row { end.row - start.row + 1 } else { start.row - end.row + 1 };
                let w = if end.col >= start.col { end.col - start.col + 1 } else { start.col - end.col + 1 };
                return Some((start.row as usize, start.col as usize, h as usize, w as usize));
            }
            Ref::Name(_) => {}
        },
        Expr::Variable(name, _) => {
            if let Some((sc, sr, ec, er)) = parse_range_ref(name) {
                let h = if er >= sr { er - sr + 1 } else { sr - er + 1 };
                let w = if ec >= sc { ec - sc + 1 } else { sc - ec + 1 };
                return Some((sr, sc, h, w));
            }
            if let Some((col, row)) = parse_cell_ref(name) {
                return Some((row, col, 1, 1));
            }
        }
        _ => {}
    }
    None
}

/// Decode a ref string (plain A1, A1:C3, or __offset__:…) into (row, col, height, width).
fn decode_ref_string_geometry(s: &str) -> Option<(usize, usize, usize, usize)> {
    if let Some((row, col, h, w)) = parse_offset_ref(s) {
        return Some((row, col, h, w));
    }
    if let Some((sc, sr, ec, er)) = parse_range_ref(s) {
        let h = if er >= sr { er - sr + 1 } else { sr - er + 1 };
        let w = if ec >= sc { ec - sc + 1 } else { sc - ec + 1 };
        return Some((sr, sc, h, w));
    }
    if let Some((col, row)) = parse_cell_ref(s) {
        return Some((row, col, 1, 1));
    }
    None
}

// ---------------------------------------------------------------------------
// ROW / COLUMN
// ---------------------------------------------------------------------------

/// `ROW([cell_ref])` — returns the row number of a cell reference.
/// Without argument, returns 1 (no row context in standalone evaluator).
pub fn row_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 0, 1) {
        return err;
    }
    if args.is_empty() {
        return Value::Number(1.0);
    }
    if let Some((row, _col, _h, _w)) = resolve_lazy_arg_geometry(&args[0], ctx) {
        return Value::Number(row as f64);
    }
    // Fallback: evaluate and forward errors.
    let v = evaluate_expr(&args[0], ctx);
    match v {
        Value::Error(e) => Value::Error(e),
        Value::ErrorMsg(e, m) => Value::ErrorMsg(e, m),
        _ => Value::Error(ErrorKind::NA),
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
    if let Some((_row, col, _h, _w)) = resolve_lazy_arg_geometry(&args[0], ctx) {
        return Value::Number(col as f64);
    }
    let v = evaluate_expr(&args[0], ctx);
    match v {
        Value::Error(e) => Value::Error(e),
        Value::ErrorMsg(e, m) => Value::ErrorMsg(e, m),
        _ => Value::Error(ErrorKind::NA),
    }
}

// ---------------------------------------------------------------------------
// ROWS / COLUMNS  (lazy — override the eager versions in the array module so
//                  that INDIRECT("A1:C5") and OFFSET(...) are handled correctly)
// ---------------------------------------------------------------------------

/// `ROWS(array_or_range)` — returns the number of rows.
pub fn rows_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, 1) {
        return err;
    }
    if let Some((_row, _col, h, _w)) = resolve_lazy_arg_geometry(&args[0], ctx) {
        return Value::Number(h as f64);
    }
    // Fall through: evaluate and count rows from the resulting Value.
    let v = evaluate_expr(&args[0], ctx);
    count_rows_from_value(&v)
}

/// `COLUMNS(array_or_range)` — returns the number of columns.
pub fn columns_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, 1) {
        return err;
    }
    if let Some((_row, _col, _h, w)) = resolve_lazy_arg_geometry(&args[0], ctx) {
        return Value::Number(w as f64);
    }
    let v = evaluate_expr(&args[0], ctx);
    count_cols_from_value(&v)
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
        Value::Error(_) | Value::ErrorMsg(_, _) => v.clone(),
        _ => Value::Number(1.0),
    }
}

fn count_cols_from_value(v: &Value) -> Value {
    match v {
        Value::Array(outer) => {
            let is_2d = outer.iter().any(|e| matches!(e, Value::Array(_)));
            if is_2d {
                match outer.first() {
                    Some(Value::Array(row)) => Value::Number(row.len() as f64),
                    _ => Value::Number(1.0),
                }
            } else {
                Value::Number(outer.len() as f64)
            }
        }
        Value::Error(_) | Value::ErrorMsg(_, _) => v.clone(),
        _ => Value::Number(1.0),
    }
}
