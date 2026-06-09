use crate::eval::functions::{check_arity, check_arity_len, EvalCtx};
use crate::eval::evaluate_expr;
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

// ── Eager versions (used by unit tests) ──────────────────────────────────────

pub fn isnumber_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    Value::Bool(matches!(args[0], Value::Number(_) | Value::Date(_)))
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
/// When given an array, checks the first element (implicit intersection).
pub fn isnumber_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    let scalar = match val {
        Value::Array(ref elems) => elems.first().cloned().unwrap_or(Value::Empty),
        other => other,
    };
    Value::Bool(matches!(scalar, Value::Number(_) | Value::Date(_)))
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
    Value::Bool(matches!(args[0], Expr::Variable(_, _) | Expr::Reference(_, _)))
}

/// `ISFORMULA(value)` — TRUE if the argument is a cell ref containing a formula.
///
/// GS semantics captured by the fixtures:
/// - An inline literal (`{1}`, `42`, `{"text"}`, `{TRUE}`) → `#N/A`.
/// - A bare cell-reference variable → `FALSE` (no formula context in the stateless
///   evaluator; we can only say the cell exists, not whether it holds a formula).
/// - `INDIRECT(<valid ref>)` → `FALSE` (same reasoning).
/// - `INDIRECT(<invalid ref>)` → `#REF!` (propagate the error).
/// - Any other call that resolves to an error → propagate that error.
pub fn isformula_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    match &args[0] {
        // Bare cell/range reference → always FALSE in stateless evaluator.
        Expr::Variable(_, _) | Expr::Reference(_, _) => Value::Bool(false),
        // Any other expression: evaluate it.
        // - Valid INDIRECT → Empty → FALSE.
        // - Invalid INDIRECT → #REF! → propagate.
        // - Inline array / literal → #N/A.
        other => {
            let val = evaluate_expr(other, ctx);
            match val {
                // Valid cell reference (e.g. INDIRECT of valid addr) returns Empty.
                Value::Empty => Value::Bool(false),
                // Errors propagate (covers #REF! from INDIRECT of invalid addr).
                Value::Error(e) => Value::Error(e),
                // Anything else (number, text, bool, array) is not a reference.
                _ => Value::Error(ErrorKind::NA),
            }
        }
    }
}

/// `ISDATE(value)` — TRUE if value is a date (typed as Date, or a valid date string).
///
/// Supported string formats (case-insensitive):
/// - ISO 8601: `YYYY-MM-DD`
/// - `"Month Day Year"` e.g. `"July 20 1969"`, `"January 1 2024"`
/// - `"Month Day"` e.g. `"July 20"` (year omitted)
///
/// Returns FALSE for:
/// - Month-only strings like `"July"`
/// - Invalid dates like `"Feb 30"`
/// - Non-text values (numbers, booleans)
pub fn isdate_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    match val {
        Value::Error(_) => val,
        Value::Date(_) => Value::Bool(true),
        Value::Text(ref s) => Value::Bool(is_date_string(s)),
        _ => Value::Bool(false),
    }
}

/// `ISEMAIL(value)` — TRUE if the argument is a valid email address string.
/// Non-text values return FALSE (not an error).
pub fn isemail_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if check_arity_len(args.len(), 1, 1).is_some() {
        return Value::Error(ErrorKind::NA);
    }
    let val = evaluate_expr(&args[0], ctx);
    match val {
        Value::Text(ref s) => Value::Bool(is_valid_email(s)),
        _ => Value::Bool(false),
    }
}

pub fn is_valid_email(s: &str) -> bool {
    // Split on '@': must have exactly one '@', non-empty local and domain parts
    let at_pos = match s.find('@') {
        Some(pos) => pos,
        None => return false,
    };
    // Ensure only one '@'
    if s[at_pos + 1..].contains('@') {
        return false;
    }
    let local = &s[..at_pos];
    let domain = &s[at_pos + 1..];
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    // Domain must have at least one dot with non-empty parts
    let domain_parts: Vec<&str> = domain.split('.').collect();
    if domain_parts.len() < 2 {
        return false;
    }
    domain_parts.iter().all(|p| !p.is_empty())
}

/// Returns `true` if `s` is a recognisable date string.
///
/// Accepted formats:
/// 1. ISO 8601: `YYYY-MM-DD`
/// 2. `"Month Day Year"` — e.g. `"July 20 1969"`, `"January 1 2024"`
/// 3. `"Month Day"` — e.g. `"July 20"` (year omitted is fine)
///
/// Returns `false` for month-only (`"July"`) and for invalid calendar dates
/// (`"Feb 30"`).
fn is_date_string(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    // Try ISO format first: YYYY-MM-DD
    if is_iso_date(s) {
        return true;
    }
    // Try natural-language formats
    is_natural_date(s)
}

/// Match ISO 8601 dates: YYYY-MM-DD (with basic calendar validation).
fn is_iso_date(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 { return false; }
    let ok_year  = parts[0].len() == 4 && parts[0].chars().all(|c| c.is_ascii_digit());
    let ok_month = parts[1].len() == 2 && parts[1].chars().all(|c| c.is_ascii_digit());
    let ok_day   = parts[2].len() == 2 && parts[2].chars().all(|c| c.is_ascii_digit());
    if !(ok_year && ok_month && ok_day) { return false; }
    let month: u32 = parts[1].parse().unwrap_or(0);
    let day: u32   = parts[2].parse().unwrap_or(0);
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

/// Month name → (month_index 1-12, max_days) or None.
fn parse_month_name(s: &str) -> Option<(u32, u32)> {
    match s.to_ascii_lowercase().as_str() {
        "january" | "jan"   => Some((1, 31)),
        "february" | "feb"  => Some((2, 29)), // allow 29 for leap years; we check via calendar
        "march" | "mar"     => Some((3, 31)),
        "april" | "apr"     => Some((4, 30)),
        "may"               => Some((5, 31)),
        "june" | "jun"      => Some((6, 30)),
        "july" | "jul"      => Some((7, 31)),
        "august" | "aug"    => Some((8, 31)),
        "september" | "sep" | "sept" => Some((9, 30)),
        "october" | "oct"   => Some((10, 31)),
        "november" | "nov"  => Some((11, 30)),
        "december" | "dec"  => Some((12, 31)),
        _ => None,
    }
}

/// Returns true if (month, day) is a valid calendar combination.
/// Uses a simple upper-bound approach; for February we allow up to 28
/// (rejecting Feb 30 etc.) but allow 29 for potential leap years.
fn is_valid_month_day(month: u32, day: u32) -> bool {
    if day == 0 { return false; }
    let max = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 29, // allow Feb 29 (leap year); reject Feb 30+
        _ => return false,
    };
    day <= max
}

/// Try to parse a natural-language date string.
///
/// Accepted tokens (space-separated):
/// - `"Month Day Year"` (3 tokens) — e.g. `"July 20 1969"`
/// - `"Month Day"` (2 tokens) — e.g. `"July 20"`
/// - `"Month"` alone (1 token) → **FALSE** (not a full date)
fn is_natural_date(s: &str) -> bool {
    try_parse_natural_date(s).unwrap_or(false)
}

fn try_parse_natural_date(s: &str) -> Option<bool> {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    match tokens.len() {
        2 => {
            // "Month Day"
            let (month, _max) = parse_month_name(tokens[0])?;
            let day: u32 = tokens[1].parse().ok()?;
            Some(is_valid_month_day(month, day))
        }
        3 => {
            // "Month Day Year"
            let (month, _max) = parse_month_name(tokens[0])?;
            let day: u32 = tokens[1].parse().ok()?;
            let _year: u32 = tokens[2].parse().ok()?; // just validate it's a number
            Some(is_valid_month_day(month, day))
        }
        _ => Some(false),
    }
}

#[cfg(test)]
mod tests;
