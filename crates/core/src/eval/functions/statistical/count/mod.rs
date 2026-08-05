use crate::eval::evaluate_expr;
use crate::eval::functions::EvalCtx;
use crate::eval::functions::date::serial::{text_to_date_serial, text_to_time_serial};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

// ── Eager versions (kept for unit tests) ─────────────────────────────────────

/// `COUNT(value1, ...)` — count of Numbers and Dates.
/// Used only in unit tests; the evaluator uses the lazy version, which is the
/// one that implements COUNT's real direct-arg / array-context split.
///
/// A `Date` counts, for the reason set out on [`count_lazy_fn`].
pub fn count_fn(args: &[Value]) -> Value {
    let n = args
        .iter()
        .filter(|v| matches!(v, Value::Number(_) | Value::Date(_)))
        .count();
    Value::Number(n as f64)
}

/// `COUNTA(value1, ...)` — count of non-empty values.
pub fn counta_fn(args: &[Value]) -> Value {
    let n = args.iter().filter(|v| !matches!(v, Value::Empty)).count();
    Value::Number(n as f64)
}

// ── Lazy versions (registered) ────────────────────────────────────────────────

/// Lazy COUNT: in direct args counts Numbers, Dates, Bool, numeric Text; in
/// arrays counts only Numbers and Dates.
/// Returns #N/A when called with no arguments.
///
/// **A date counts.** In Google Sheets a date is a serial number carrying a
/// display format, and COUNT counts it wherever it appears — as a direct
/// argument, inside an array literal, and through a range. Captured on the
/// conformance-fixtures pipeline (2026-08-04, locale `en_US`, timezone
/// `Etc/GMT`), with the control `=COUNT(5)` → 1 proving the probe resolved:
///
/// ```text
/// =COUNT(DATE(2020,1,1))                     1
/// =COUNT(DATE(2020,1,1),DATE(2021,1,1))      2
/// =COUNT({DATE(2020,1,1),DATE(2021,1,1)})    2
/// =COUNT({DATE(2020,1,1),5})                 2
/// =COUNT(<a range of three dates>)           3
/// =COUNT(<a range of two dates and a 5>)     3
/// =COUNT(MAX(<a range of three dates>))      1
/// =COUNT({DATE(2020,1,1),TRUE})              1
/// =COUNT({DATE(2020,1,1),"a"})               1
/// ```
///
/// Sibling extremes were probed alongside it: `=COUNT(MIN(...))`,
/// `=COUNT(MAXA(...))`, `=COUNT(MINA(...))` and `=COUNT(LARGE(...,1))` over a
/// date range are all 1.
///
/// **None of those rows are in this repo yet.** They come off the
/// conformance-fixtures pipeline and land in a separate fixtures-only PR — CI
/// rejects a PR that mixes fixture TSVs with code. A reviewer working from
/// this repo alone can check the unit tests and this comment; the Sheets
/// answers themselves have to be taken from that pipeline.
///
/// Counting a date changes nothing else. The last two captured rows above say
/// so directly, and they agree with what `statistical.tsv` already records for
/// booleans and text on their own (`=COUNT(TRUE,FALSE,1)` is 3,
/// `=COUNT({TRUE,FALSE,1,2})` is 2, `=COUNT({"1","2","3"})` is 0): a date
/// beside a boolean or a string does not pull either into an array's scope.
///
/// Two gaps this probe found are **not** fixed here:
///
/// - `=COUNT("2020-01-01")` is 1 in Sheets and 0 here. That is a rule about
///   *text* — Sheets reads a date- or time-shaped string as a number in direct
///   args, while this arm counts only text that parses as an `f64` — not a
///   rule about `Date` values. Array context already agrees. See #790.
/// - `SUBTOTAL`'s collector and `DCOUNT` drop a `Date` through the same shape
///   of catch-all this function just lost. See #792.
pub fn count_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut n = 0usize;
    for arg in args {
        count_direct(&evaluate_expr(arg, ctx), &mut n);
    }
    Value::Number(n as f64)
}

/// Whether a direct text argument is one Sheets reads as a number.
///
/// Sheets counts direct text it can read as a number *or* as a date or time,
/// which the captured rows spell out: ISO dates, `M/D/YYYY`, `D-MMM-YYYY`,
/// times with and without seconds, AM/PM, datetimes, and any of those with
/// surrounding whitespace. `"2020-13-01"` and `"abc"` are not counted, so this
/// is real parsing rather than a shape match.
///
/// Deliberately narrower than `VALUE()`, which also accepts `"12%"`, `"$42"`
/// and `"1,234.56"`. Nothing recorded says whether Sheets counts those, so
/// they stay out until they are probed.
fn reads_as_number_or_datetime(s: &str) -> bool {
    let trimmed = s.trim();
    // Empty text is not a number: `=COUNT("")` is 0, and the time parser would
    // otherwise accept it.
    if trimmed.is_empty() {
        return false;
    }
    trimmed.parse::<f64>().is_ok()
        || text_to_date_serial(trimmed).is_some()
        || text_to_time_serial(trimmed).is_some()
}

/// COUNT's direct-argument rules.
///
/// Every `Value` variant is spelled out rather than swept up by a catch-all,
/// so a variant added later is a compile error here instead of being silently
/// uncounted — which is exactly how `Date` came to be missed.
fn count_direct(v: &Value, n: &mut usize) {
    match v {
        Value::Array(elems) => {
            for elem in elems {
                count_in_array(elem, n);
            }
        }
        Value::Number(_) => *n += 1,
        // A date is a serial number with a display format, and Sheets counts
        // it. See [`count_lazy_fn`] for the captured rows.
        Value::Date(_) => *n += 1,
        Value::Bool(_) => *n += 1,
        Value::Text(s) if reads_as_number_or_datetime(s) => *n += 1,
        Value::Text(_) => {}
        // A zone-aware instant carries no serial and is not a number anywhere
        // else in this family either — every statistical collector treats it
        // as non-numeric. Uncounted, as it has always been; there is no Sheets
        // equivalent to probe.
        Value::Zoned(_) => {}
        Value::Empty | Value::Sparkline(_) | Value::Error(_) | Value::ErrorMsg(_, _) => {}
    }
}

/// COUNT's array-context rules: only Numbers and Dates count — booleans and
/// text do not, which `statistical.tsv` records (`=COUNT({TRUE,FALSE,1,2})` is
/// 2, `=COUNT({"1","2","3"})` is 0).
///
/// Exhaustive for the same reason as [`count_direct`].
fn count_in_array(v: &Value, n: &mut usize) {
    match v {
        Value::Array(elems) => {
            for elem in elems {
                count_in_array(elem, n);
            }
        }
        Value::Number(_) => *n += 1,
        Value::Date(_) => *n += 1,
        Value::Bool(_) | Value::Text(_) => {}
        Value::Zoned(_) => {}
        Value::Empty | Value::Sparkline(_) | Value::Error(_) | Value::ErrorMsg(_, _) => {}
    }
}

/// Lazy COUNTA: counts everything that is not Empty (including errors).
/// Arrays are flattened: each non-empty element is counted individually.
/// Returns #N/A when called with no arguments.
pub fn counta_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut n = 0usize;
    for arg in args {
        count_non_empty(&evaluate_expr(arg, ctx), &mut n);
    }
    Value::Number(n as f64)
}

/// COUNTA's rule: everything that is not blank counts, errors included.
///
/// Spelled out rather than left as a catch-all so a variant added later is a
/// compile error here too — the rule below is "count it", but that is a
/// decision to take deliberately for a new kind of value rather than inherit.
fn count_non_empty(v: &Value, n: &mut usize) {
    match v {
        Value::Array(elems) => {
            for elem in elems {
                count_non_empty(elem, n);
            }
        }
        Value::Empty => {}
        Value::Number(_)
        | Value::Date(_)
        | Value::Bool(_)
        | Value::Text(_)
        | Value::Zoned(_)
        | Value::Sparkline(_)
        | Value::Error(_)
        | Value::ErrorMsg(_, _) => *n += 1,
    }
}

#[cfg(test)]
mod tests;
