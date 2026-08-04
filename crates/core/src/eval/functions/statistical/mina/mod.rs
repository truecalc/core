use crate::types::{ErrorKind, Value};

/// `MINA(value1, ...)` — smallest value, coercing booleans (TRUE=1, FALSE=0).
/// - Numbers included directly.
/// - Booleans coerced: TRUE=1, FALSE=0.
/// - Text in direct args → `#VALUE!`.
/// - Empty → skip.
/// - Empty array argument → `#REF!`.
/// - Array of nothing but blanks → 0.
/// - No args → `#N/A`.
///
/// **Dates participate as bare serials** and carry their type out, exactly as
/// they do for MIN: a date-only range answers the earliest date, a date beside
/// a plain number is compared on the serial (so a small plain number beats
/// every date), and the result is date-typed whenever a date took part — even
/// when the plain number won.
///
/// Captured alongside the MAX/MIN forms, which agree with MINA on every date
/// input — but captured and extrapolated are not the same thing here:
///
/// - **Values** are captured in Google Sheets for every form — a date-only
///   column, a date/number column, array literals of both shapes, and dates
///   passed as direct arguments.
/// - **Typing** is captured for the *range* forms only, read back through the
///   cell that holds the result. The literal and direct-argument rows report
///   `number`, but that is an artifact of the capture harness reading them
///   through an `INDEX(...,1,1)` wrapper, which drops the cell's date format —
///   it is not a Sheets answer. So the date typing of
///   `=MINA({DATE(...),DATE(...)})` is **extrapolated** from the range forms,
///   not probed.
///
/// **None of those rows are in this repo yet.** They come off the
/// conformance-fixtures pipeline and land in a separate fixtures-only PR —
/// they fail until this code exists, and CI rejects a PR that mixes fixture
/// TSVs with code. Same arrangement as the blank-only rows described on
/// `stat_helpers::is_blank_only_array`. A reviewer working from this repo
/// alone can check the unit tests and this comment; the Sheets answers
/// themselves have to be taken from that pipeline.
///
/// # Zone-aware values (#781)
///
/// MINA consults [`stat_helpers::zoned_extreme`] before its argument loop,
/// exactly as MIN does: zoned instants on their own answer the earliest,
/// preserving its zone; a zoned instant mixed with anything that contributes a
/// number — `Number`, `Date`, `Bool` or `Text` — is `#VALUE!`.
///
/// This is a **deliberate truecalc-only decision, not a captured Sheets
/// answer** — `Zoned` has no Sheets equivalent, so the conformance oracle
/// cannot settle it. The reasoning is set out in full on [`maxa_fn`]; the same
/// rule applies to all four of MAX, MIN, MAXA and MINA so that the answer does
/// not depend on which member of the family is called.
///
/// The check runs **before** the argument loop, so it precedes the empty-array
/// `#REF!`, the sparkline-only 0 and the blank-only 0 — the same ordering MIN
/// has always had.
///
/// [`stat_helpers::zoned_extreme`]: super::stat_helpers::zoned_extreme
/// [`maxa_fn`]: super::maxa::maxa_fn
pub fn mina_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // Zone-aware participation, decided identically to MIN — see `maxa_fn` for
    // why the A-variants do not get their own rule.
    if let Some(r) = super::stat_helpers::zoned_extreme(args, true) {
        return r;
    }
    let mut result: Option<f64> = None;
    // See `fold_array_min` for why this flag exists.
    let mut skipped_sparkline = false;
    let mut saw_date = false;
    for arg in args {
        match arg {
            Value::Sparkline(_) => skipped_sparkline = true,
            Value::Number(n) => {
                result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
            }
            Value::Date(n) => {
                saw_date = true;
                result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
            }
            Value::Bool(b) => {
                let n = if *b { 1.0 } else { 0.0 };
                result = Some(result.map_or(n, |cur: f64| cur.min(n)));
            }
            Value::Text(_) => return Value::Error(ErrorKind::Value),
            Value::Empty => {}
            Value::Array(inner) => {
                // An empty argument is #REF!, as it is for MIN and MAX
                // (`=MINA({})`). Note MINA reaches the *other* answers by a
                // different route: text folds in as 0 rather than being
                // skipped, so `=MINA({"a","b"})` is already 0 without any
                // "populated but numberless" rule.
                if inner.is_empty() {
                    return Value::Error(ErrorKind::Ref);
                }
                // In array context: Numbers included, Dates included as their
                // bare serial (and they type the answer), Bool→1/0, Text→0,
                // Empty→skip.
                // Recurses into nested arrays (e.g. a vertical range
                // materializes as nested one-element row arrays).
                if let Err(e) =
                    fold_array_min(inner, &mut result, &mut skipped_sparkline, &mut saw_date)
                {
                    return e;
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
            // Listed rather than a catch-all so a new `Value` variant is a
            // compile error here instead of a silent skip. A `Zoned` only
            // reaches this loop when no argument was zone-aware, which
            // `zoned_extreme` above has already ruled on.
            Value::Zoned(_) => {}
        }
    }
    match result {
        // A date anywhere in scope makes the answer date-typed, whether or not
        // the date is the value that won.
        Some(n) if saw_date => Value::Date(n),
        Some(n) => Value::Number(n),
        None if skipped_sparkline => Value::Number(0.0),
        // An array of nothing but blanks is 0, not #N/A: `=MINA(A1:A3)` over
        // empty cells answers the same 0 that MIN, MAX and MAXA give it — see
        // `is_blank_only_array` for every range shape that was probed, the
        // controls that prove the ranges resolved, and where the rows live. A
        // blank argument with no array in sight keeps the #N/A below — that
        // shape is unprobed.
        None if super::stat_helpers::is_blank_only_array(args) => Value::Number(0.0),
        None    => Value::Error(ErrorKind::NA),
    }
}

/// Recurse into nested arrays (e.g. a vertical range materializes as nested
/// one-element row arrays) so every cell is visited, folding into `result`
/// with MINA's array-context coercion rules.
/// A sparkline is skipped wherever it appears, and an aggregate whose scope
/// holds nothing else answers 0 — the same answer whether it arrived as a
/// direct argument or through a range (google.tsv: `=MINA(SPARKLINE({1,2,3}))`
/// and `=MINA(Data!K1:K1)` are both 0). The flag is what distinguishes "skipped a
/// sparkline" from "saw nothing usable at all", which stay different answers.
fn fold_array_min(
    arr: &[Value],
    result: &mut Option<f64>,
    skipped_sparkline: &mut bool,
    saw_date: &mut bool,
) -> Result<(), Value> {
    for v in arr {
        let n = match v {
            Value::Sparkline(_) => {
                *skipped_sparkline = true;
                continue;
            }
            Value::Number(n) => *n,
            // A date folds in as its bare serial and types the answer.
            Value::Date(n) => {
                *saw_date = true;
                *n
            }
            Value::Bool(b) => if *b { 1.0 } else { 0.0 },
            Value::Text(_) => 0.0,
            Value::Empty => continue,
            Value::Array(inner) => {
                fold_array_min(inner, result, skipped_sparkline, saw_date)?;
                continue;
            }
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            // Listed rather than a catch-all so a new `Value` variant is a
            // compile error here instead of inheriting "skipped" by accident.
            Value::Zoned(_) => continue,
        };
        *result = Some(result.map_or(n, |cur: f64| cur.min(n)));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
