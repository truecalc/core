use crate::eval::evaluate_expr;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// `FORMULATEXT(reference)` -- returns the formula string of a cell reference.
/// In a standalone evaluator, there is no cell grid, so most calls return #N/A.
/// Exception: if the argument evaluates to a #REF! error, propagate #REF! instead.
pub fn formulatext_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 1, 1) {
        return err;
    }
    // Evaluate the argument to propagate errors (especially #REF!)
    let val = evaluate_expr(&args[0], ctx);
    match val {
        Value::Error(e) => Value::Error(e),
        Value::ErrorMsg(e, m) => Value::ErrorMsg(e, m),
        _ => Value::Error(ErrorKind::NA),
    }
}

/// `GETPIVOTDATA(data_field, pivot_table, ...)` -- always errors in a standalone evaluator.
/// With 0 or 1 args -> #N/A (wrong arity). With 2+ args -> #REF! (no pivot table present).
pub fn getpivotdata_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    // Evaluate the pivot_table arg (2nd arg) -- if it is already an error, propagate it.
    let pivot_ref = evaluate_expr(&args[1], ctx);
    if pivot_ref.is_error() {
        return pivot_ref;
    }
    Value::Error(ErrorKind::Ref)
}

/// Encode an offset result as a special text tag that downstream lazy functions
/// (ROW/COLUMN/ROWS/COLUMNS) can decode.
/// Format: `__offset__:<row>:<col>:<height>:<width>` (all 1-based, >= 1).
pub fn make_offset_tag(row: i64, col: i64, height: i64, width: i64) -> Value {
    Value::Text(format!(
        "__offset__:{}:{}:{}:{}",
        row, col, height.abs(), width.abs()
    ))
}

/// Parse an `__offset__:row:col:height:width` tag.
/// Returns `(row, col, height, width)` -- all positive, 1-based.
pub fn parse_offset_ref(s: &str) -> Option<(usize, usize, usize, usize)> {
    let s = s.strip_prefix("__offset__:")?;
    let mut parts = s.splitn(4, ':');
    let row:    usize = parts.next()?.parse().ok()?;
    let col:    usize = parts.next()?.parse().ok()?;
    let height: usize = parts.next()?.parse().ok()?;
    let width:  usize = parts.next()?.parse().ok()?;
    if row == 0 || col == 0 { return None; }
    Some((row, col, height, width))
}

/// `OFFSET(reference, rows, cols, [height], [width])` -- not implementable for value
/// retrieval without a cell grid, but we compute the resulting address and encode it
/// as `__offset__:row:col:height:width` so that ROW/COLUMN/ROWS/COLUMNS can use it.
/// SUM(OFFSET(...)) yields 0 (grid is empty).
pub fn offset_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    if let Some(err) = check_arity_len(args.len(), 3, 5) {
        return err;
    }

    let ref_val   = evaluate_expr(&args[0], ctx);
    let rows_val  = evaluate_expr(&args[1], ctx);
    let cols_val  = evaluate_expr(&args[2], ctx);
    let height_val = if args.len() >= 4 { Some(evaluate_expr(&args[3], ctx)) } else { None };
    let width_val  = if args.len() >= 5 { Some(evaluate_expr(&args[4], ctx)) } else { None };

    // Propagate errors from mandatory args.
    for v in [&ref_val, &rows_val, &cols_val] {
        if v.is_error() { return v.clone(); }
    }
    if let Some(v) = &height_val { if v.is_error() { return v.clone(); } }
    if let Some(v) = &width_val  { if v.is_error() { return v.clone(); } }

    let row_offset = match rows_val {
        Value::Number(n) => n.trunc() as i64,
        _ => return Value::Error(ErrorKind::Value),
    };
    let col_offset = match cols_val {
        Value::Number(n) => n.trunc() as i64,
        _ => return Value::Error(ErrorKind::Value),
    };

    // height / width default to 1; 0 -> #VALUE!
    let height: i64 = match height_val {
        None => 1,
        Some(Value::Number(n)) => {
            let h = n.trunc() as i64;
            if h == 0 { return Value::Error(ErrorKind::Value); }
            h
        }
        _ => 1,
    };
    let width: i64 = match width_val {
        None => 1,
        Some(Value::Number(n)) => {
            let w = n.trunc() as i64;
            if w == 0 { return Value::Error(ErrorKind::Value); }
            w
        }
        _ => 1,
    };

    // Resolve the base reference to (row, col).
    let (base_row, base_col) = match ref_position_from_value(&ref_val) {
        Some(pos) => pos,
        None => return Value::Error(ErrorKind::Ref),
    };

    let new_row = base_row + row_offset;
    let new_col = base_col + col_offset;

    if new_row < 1 || new_col < 1 {
        return Value::Error(ErrorKind::Ref);
    }

    make_offset_tag(new_row, new_col, height, width)
}

/// Extract (row, col) from an already-evaluated value.
/// Handles the `__offset__:...` tag and cell reference strings.
pub fn ref_position_from_value(v: &Value) -> Option<(i64, i64)> {
    match v {
        Value::Text(s) => {
            if let Some((r, c, _, _)) = parse_offset_ref(s) {
                return Some((r as i64, c as i64));
            }
            use super::cell_ref::{parse_cell_ref, parse_range_ref};
            if let Some((sc, sr, _, _)) = parse_range_ref(s) {
                return Some((sr as i64, sc as i64));
            }
            if let Some((col, row)) = parse_cell_ref(s) {
                return Some((row as i64, col as i64));
            }
            None
        }
        _ => None,
    }
}

/// `SHEET([name])` -- returns the sheet index.
/// With no argument, returns 1 (standalone evaluator has 1 sheet).
/// With a text argument -> #REF! (sheet not found).
pub fn sheet_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    match args.len() {
        0 => Value::Number(1.0),
        1 => {
            let val = evaluate_expr(&args[0], ctx);
            match val {
                Value::Text(_) => Value::Error(ErrorKind::Ref),
                Value::Error(e) => Value::Error(e),
                Value::ErrorMsg(e, m) => Value::ErrorMsg(e, m),
                _ => Value::Error(ErrorKind::NA),
            }
        }
        _ => Value::Error(ErrorKind::NA),
    }
}
