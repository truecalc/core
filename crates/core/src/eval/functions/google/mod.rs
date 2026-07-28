//! `SPARKLINE(data, [options])` — the in-cell chart function.
//!
//! The engine's job here is parsing and validation, not drawing: a successful
//! call produces a [`Value::Sparkline`] carrying the parsed, validated
//! [`SparklineSpec`], and the consumer renders it.
//!
//! ## What Google Sheets does (conformance fixtures, `google.tsv`)
//!
//! The result is a value kind of its own — `TYPE()` reports `128` (outside
//! `TYPE`'s documented set) and `ISERROR()` is `FALSE`.
//!
//! ## Coercion — where the answers live
//!
//! Three coercion seams carry most contexts, each a single blanket arm in
//! [`crate::eval::coercion`] that every caller of that seam inherits:
//!
//! | seam | a sparkline reads as |
//! |---|---|
//! | `to_number` | `#VALUE!` |
//! | `to_string_val` | `""` |
//! | `to_bool` | `false` |
//!
//! **That is not enough to predict any particular function.** Individual
//! functions carry hand-carved arms that override their seam — `N`, `TEXT`, the
//! `TO_*` family, the `&` operator, and the aggregates — and they are spread
//! across the codebase, so no list here can stay complete. (The aggregates are
//! uniform *as of the rows below*: every one of them skips a sparkline and
//! answers 0 when nothing else is in scope, `AVERAGE` alone answering `#DIV/0!`.
//! That uniformity is a fact about the probed set, not a rule to extend from —
//! `MINA` had no arm at all until a row asked for one.)
//!
//! The record is `tests/fixtures/google_sheets/google.tsv`, every row of it
//! observed in live Google Sheets and the only authority here. This function
//! contributes **114** of that file's **128** data rows (the other 14 are
//! ARRAYFORMULA's, and a 115th recorded row sits in `bugs.tsv` — see below).
//! `tests/sparkline.rs` asserts the rows the TSV runner skips.
//!
//! One recorded row is a known engine divergence rather than a passing case:
//! `=MIN(SPARKLINE({1,2,3}),{})` is `#REF!` in Sheets and `0` here, because
//! `MIN` has no empty-array rule at all (`=MIN({})` is `0` on main, while
//! `=MAX({})` is `#REF!`). That gap predates sparklines and is unrelated to
//! them, so it lives in `bugs.tsv` awaiting its own fix rather than being
//! patched from here.
//!
//! Why not reason it out instead: `DOLLAR(sparkline)` is `#VALUE!` while
//! `TO_DOLLARS(sparkline)` is `""`. Near-identical names, both format a number
//! as currency text, opposite answers.
//!
//! So: **if you need the answer for a function, find its row. If it has no row,
//! probe it.** Do not infer one from its seam or from a function that resembles
//! it.
//!
//! Validation splits into three distinct error classes:
//!
//! - `#N/A` — the arity/shape of `data`: no arguments, a scalar instead of a
//!   range, or a single value.
//! - `#REF!` — structural malformation: an empty array, or an `options` array
//!   that is not key/value pairs.
//! - `#VALUE!` — a bad option *value* (an unrecognised `charttype`).
//!
//! An unrecognised option **key** is *not* an error — Sheets ignores it, which
//! is what lets a workbook written against a newer option set still evaluate.
//! A genuine blank cell inside the source range is likewise fine: it renders.

use crate::display::display_number;
use crate::eval::functions::{check_arity, FunctionMeta, Registry};
use crate::types::{ErrorKind, SparklineChartType, SparklineSpec, SparklineValue, Value};

/// The `charttype` option key, lifted out of the generic option list.
const CHART_TYPE_KEY: &str = "charttype";

/// Convert one evaluated cell into a plotted point / option value.
///
/// `Zoned` and a nested sparkline have no Google Sheets analogue, so there is no
/// ground truth for them; they are rejected with `#VALUE!` rather than given an
/// invented projection. Those two arms are reachable only for a cell *inside*
/// an array — a sparkline handed straight to `data` never gets here, because
/// [`parse_data`]'s non-array check answers `#N/A` first (the same answer as any
/// other scalar `data` argument).
fn to_sparkline_value(v: &Value) -> Result<SparklineValue, Value> {
    match v {
        Value::Number(n) | Value::Date(n) => Ok(SparklineValue::number(*n)),
        Value::Text(s) => Ok(SparklineValue::Text(s.clone())),
        Value::Bool(b) => Ok(SparklineValue::Bool(*b)),
        Value::Empty => Ok(SparklineValue::Blank),
        Value::Error(_) | Value::ErrorMsg(_, _) => Err(v.clone()),
        Value::Array(_) | Value::Zoned(_) | Value::Sparkline(_) => {
            Err(Value::Error(ErrorKind::Value))
        }
    }
}

/// Flatten a (possibly row-nested) array into its cells, row-major.
fn flatten<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
    match v {
        Value::Array(elems) => elems.iter().for_each(|e| flatten(e, out)),
        other => out.push(other),
    }
}

/// Parse and validate the `data` argument.
///
/// `#N/A` for a scalar or a single value, `#REF!` for an empty array.
fn parse_data(v: &Value) -> Result<Vec<SparklineValue>, Value> {
    if !matches!(v, Value::Array(_)) {
        return Err(Value::Error(ErrorKind::NA));
    }
    let mut cells = Vec::new();
    flatten(v, &mut cells);
    match cells.len() {
        0 => return Err(Value::Error(ErrorKind::Ref)),
        1 => return Err(Value::Error(ErrorKind::NA)),
        _ => {}
    }
    cells.iter().map(|c| to_sparkline_value(c)).collect()
}

/// The option-key projection of a cell.  Keys are matched case-insensitively,
/// so they are stored ASCII-lower-cased.
fn option_key(v: &Value) -> Result<String, Value> {
    let key = match v {
        Value::Text(s) => s.clone(),
        Value::Number(n) | Value::Date(n) => display_number(*n),
        Value::Bool(b) => (if *b { "TRUE" } else { "FALSE" }).to_string(),
        Value::Empty => String::new(),
        Value::Error(_) | Value::ErrorMsg(_, _) => return Err(v.clone()),
        Value::Array(_) | Value::Zoned(_) | Value::Sparkline(_) => {
            return Err(Value::Error(ErrorKind::Ref))
        }
    };
    Ok(key.to_ascii_lowercase())
}

/// Split the `options` argument into key/value pairs.
///
/// Anything that is not a well-formed pair list — a scalar, an empty array, a
/// row that is not two cells wide, an odd-length flat array — is `#REF!`.
fn option_pairs(v: &Value) -> Result<Vec<(&Value, &Value)>, Value> {
    let Value::Array(elems) = v else {
        return Err(Value::Error(ErrorKind::Ref));
    };
    if elems.is_empty() {
        return Err(Value::Error(ErrorKind::Ref));
    }
    let rows = elems.iter().filter(|e| matches!(e, Value::Array(_))).count();
    if rows == elems.len() {
        // `{"charttype","line";"color","red"}` — one key/value pair per row.
        let mut pairs = Vec::with_capacity(elems.len());
        for row in elems {
            let Value::Array(cells) = row else { unreachable!() };
            if cells.len() != 2 {
                return Err(Value::Error(ErrorKind::Ref));
            }
            pairs.push((&cells[0], &cells[1]));
        }
        Ok(pairs)
    } else if rows == 0 {
        // `{"bogus","x"}` — a single flat row of key/value cells.
        if elems.len() % 2 != 0 {
            return Err(Value::Error(ErrorKind::Ref));
        }
        Ok(elems.chunks(2).map(|p| (&p[0], &p[1])).collect())
    } else {
        // Rows mixed with bare cells is not a pair list.
        Err(Value::Error(ErrorKind::Ref))
    }
}

/// `SPARKLINE(data, [options])` — build the render spec for an in-cell chart.
pub fn sparkline_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }

    let data = match parse_data(&args[0]) {
        Ok(data) => data,
        Err(e) => return e,
    };

    let mut chart_type = SparklineChartType::Line;
    let mut options = Vec::new();
    if let Some(raw_options) = args.get(1) {
        let pairs = match option_pairs(raw_options) {
            Ok(pairs) => pairs,
            Err(e) => return e,
        };
        for (raw_key, raw_value) in pairs {
            let key = match option_key(raw_key) {
                Ok(key) => key,
                Err(e) => return e,
            };
            let value = match to_sparkline_value(raw_value) {
                Ok(value) => value,
                Err(e) => return e,
            };
            if key == CHART_TYPE_KEY {
                // A bad option *value* is `#VALUE!` (a bad option *key* is not
                // an error at all — it lands in `options` untouched).
                let SparklineValue::Text(ref name) = value else {
                    return Value::Error(ErrorKind::Value);
                };
                match SparklineChartType::parse(name) {
                    Some(t) => chart_type = t,
                    None => return Value::Error(ErrorKind::Value),
                }
            } else {
                options.push((key, value));
            }
        }
    }

    Value::Sparkline(Box::new(SparklineSpec {
        chart_type,
        data,
        options,
    }))
}

pub fn register_google(registry: &mut Registry) {
    registry.register_eager(
        "SPARKLINE",
        sparkline_fn,
        FunctionMeta {
            category: "google",
            signature: "SPARKLINE(data, [options])",
            description: "Miniature in-cell chart over a range, as a render spec",
        },
    );
}
