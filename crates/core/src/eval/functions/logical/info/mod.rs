use crate::eval::{evaluate_expr, functions::{check_arity_len, EvalCtx}};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};
use crate::eval::functions::lookup::cell_ref::{col_index_to_label, parse_cell_ref};

/// `SHEETS([reference])` — returns the number of sheets in a reference or the workbook.
/// In the standalone evaluator there is always exactly 1 sheet.
/// With no argument, returns 1.
/// With a cell-reference argument, returns 1.
/// With a non-reference argument (number, text, etc.) returns `#N/A`.
pub fn sheets_fn(args: &[Expr], _ctx: &mut EvalCtx<'_>) -> Value {
    match args.len() {
        0 => Value::Number(1.0),
        1 => {
            if matches!(args[0], Expr::Variable(_, _) | Expr::Reference(_, _)) {
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

// ---------------------------------------------------------------------------
// Helpers for CELL
// ---------------------------------------------------------------------------

/// Try to extract a cell-address string from an AST node.
/// - `Expr::Variable("B1", _)` → `Some("B1")`
/// - `Expr::FunctionCall { name: "INDIRECT", args: [Expr::Text(s, _)], .. }` → `Some(s)`
/// - Anything else → `None` (caller should evaluate the expr to check for errors)
fn extract_ref_address<'a>(expr: &'a Expr) -> Option<&'a str> {
    match expr {
        Expr::Variable(name, _) => Some(name.as_str()),
        Expr::FunctionCall { name, args, .. } if name.eq_ignore_ascii_case("INDIRECT") => {
            if args.len() >= 1 {
                if let Expr::Text(s, _) = &args[0] {
                    return Some(s.as_str());
                }
            }
            None
        }
        _ => None,
    }
}

/// `CELL(info_type, [reference])` — returns information about a cell.
///
/// Supported info_type values (case-insensitive):
/// - `"type"`: `"b"` for blank, `"l"` for label (text or empty via INDIRECT), `"n"` for number.
///   Non-reference arguments return `#N/A`.
/// - `"col"`: column number (1-based) of the reference.
/// - `"row"`: row number (1-based) of the reference.
/// - `"address"`: absolute address string like `$A$1`.
/// - `"contents"`: value of the cell. Without a live grid, valid refs return Empty.
///   Non-reference arguments return `#N/A`.
///
/// Without a second argument, or with an unrecognised info_type, returns `#N/A`.
///
/// Notes on GS semantics captured in the fixtures:
/// - An inline array literal (e.g. `{1}`) is **not** a cell reference → `#N/A`.
/// - `INDIRECT("A1")` on an empty cell evaluates to `Value::Empty`; GS types that
///   cell as `"l"` (label), not `"b"` (blank).  The fixture confirms expected = `"l"`.
pub fn cell_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    // Need at least 1 argument (info_type).
    if args.is_empty() || args.len() > 2 {
        return Value::Error(ErrorKind::NA);
    }

    // Evaluate info_type.
    let info_type_val = evaluate_expr(&args[0], ctx);
    let info_type = match &info_type_val {
        Value::Text(s) => s.to_ascii_lowercase(),
        Value::Error(e) => return Value::Error(e.clone()),
        _ => return Value::Error(ErrorKind::NA),
    };

    // Without a second argument, return #N/A (requires sheet context).
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }

    let ref_expr = &args[1];

    match info_type.as_str() {
        "type" => {
            // Non-reference (inline array, literal number, etc.) → #N/A.
            // INDIRECT(<valid>) → empty cell → GS returns "l".
            // INDIRECT(<invalid>) → #REF! (propagate).
            let val = evaluate_expr(ref_expr, ctx);
            match val {
                Value::Empty        => Value::Text("l".to_string()),
                Value::Text(_)      => Value::Text("l".to_string()),
                Value::Number(_) | Value::Date(_) => Value::Text("n".to_string()),
                Value::Bool(_)      => Value::Text("l".to_string()),
                Value::Error(e)     => Value::Error(e),
                Value::Array(_)     => Value::Error(ErrorKind::NA),
            }
        }
        "col" => {
            // Extract the reference address and parse its column.
            match extract_ref_address(ref_expr) {
                Some(addr) => {
                    if let Some((col, _row)) = parse_cell_ref(addr) {
                        Value::Number(col as f64)
                    } else {
                        // The address string exists but isn't a valid cell ref.
                        let val = evaluate_expr(ref_expr, ctx);
                        match val {
                            Value::Error(e) => Value::Error(e),
                            _ => Value::Error(ErrorKind::NA),
                        }
                    }
                }
                None => {
                    // Non-ref expression: evaluate to surface errors, else #N/A.
                    let val = evaluate_expr(ref_expr, ctx);
                    match val {
                        Value::Error(e) => Value::Error(e),
                        _ => Value::Error(ErrorKind::NA),
                    }
                }
            }
        }
        "row" => {
            match extract_ref_address(ref_expr) {
                Some(addr) => {
                    if let Some((_col, row)) = parse_cell_ref(addr) {
                        Value::Number(row as f64)
                    } else {
                        let val = evaluate_expr(ref_expr, ctx);
                        match val {
                            Value::Error(e) => Value::Error(e),
                            _ => Value::Error(ErrorKind::NA),
                        }
                    }
                }
                None => {
                    let val = evaluate_expr(ref_expr, ctx);
                    match val {
                        Value::Error(e) => Value::Error(e),
                        _ => Value::Error(ErrorKind::NA),
                    }
                }
            }
        }
        "address" => {
            match extract_ref_address(ref_expr) {
                Some(addr) => {
                    if let Some((col, row)) = parse_cell_ref(addr) {
                        let col_label = col_index_to_label(col);
                        Value::Text(format!("${col_label}${row}"))
                    } else {
                        let val = evaluate_expr(ref_expr, ctx);
                        match val {
                            Value::Error(e) => Value::Error(e),
                            _ => Value::Error(ErrorKind::NA),
                        }
                    }
                }
                None => {
                    let val = evaluate_expr(ref_expr, ctx);
                    match val {
                        Value::Error(e) => Value::Error(e),
                        _ => Value::Error(ErrorKind::NA),
                    }
                }
            }
        }
        "contents" => {
            // Only meaningful for a real cell reference; non-refs → #N/A.
            // With no live grid, a valid ref returns Empty.
            if extract_ref_address(ref_expr).is_some() {
                let val = evaluate_expr(ref_expr, ctx);
                match val {
                    Value::Error(e) => Value::Error(e),
                    other => other,
                }
            } else {
                // Non-reference (inline array, literal, etc.) → #N/A.
                let val = evaluate_expr(ref_expr, ctx);
                match val {
                    Value::Error(e) => Value::Error(e),
                    _ => Value::Error(ErrorKind::NA),
                }
            }
        }
        _ => Value::Error(ErrorKind::NA),
    }
}

#[cfg(test)]
mod tests;
