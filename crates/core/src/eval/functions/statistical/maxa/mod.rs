use crate::types::{ErrorKind, Value};

/// `MAXA(value1, ...)` — largest value, coercing booleans (TRUE=1, FALSE=0).
/// - Numbers included directly.
/// - Booleans coerced: TRUE=1, FALSE=0.
/// - Text in direct args → `#VALUE!`.
/// - Empty → skip.
/// - Empty array argument → `#REF!`.
/// - Array of nothing but blanks → 0.
/// - No args → `#N/A`.
///
/// **Dates participate as bare serials** and carry their type out, exactly as
/// they do for MAX: a date-only range answers the latest date, a date beside a
/// plain number is compared on the serial, and the result is date-typed
/// whenever a date took part — even when a plain number won.
///
/// Captured alongside the MAX/MIN forms, which agree with MAXA on every date
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
///   `=MAXA({DATE(...),DATE(...)})` is **extrapolated** from the range forms,
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
/// MAXA consults [`stat_helpers::zoned_extreme`] before its argument loop,
/// exactly as MAX does: zoned instants on their own answer the latest,
/// preserving its zone; a zoned instant mixed with anything that contributes a
/// number — `Number`, `Date`, `Bool` or `Text` — is `#VALUE!`.
///
/// This is a **deliberate truecalc-only decision, not a captured Sheets
/// answer.** `Zoned` is a truecalc extension with no Sheets equivalent, so the
/// conformance oracle cannot settle it. Two answers were defensible — error
/// like the siblings, or teach all four to compare a zoned instant against a
/// naive serial — and the first was chosen because:
///
/// - MAX and MIN already answer `#VALUE!` here, and `MAXA(<zoned>, <date>)`
///   answering a confident date while `MAX` of the same arguments errors is a
///   difference no caller can predict from the function name;
/// - the alternative requires inventing a naive-vs-aware comparison rule with
///   no ground truth behind it, which is strictly more invention;
/// - the collectors that were checked already refuse a zoned instant —
///   `collect_nums_direct`, `collect_nums_a_direct` and `collect_nums_a_checked`
///   all return `#VALUE!` — so erroring is not a new rule for this family.
///   That is three collectors, not all of them: several others still skip a
///   `Zoned` silently, and squaring them up is out of scope here;
/// - the behaviour it replaces was the failure mode both #780 and #781 are
///   about — a zone-aware value silently dropped, leaving a plausible-looking
///   number in place of a visible error.
///
/// The check runs **before** the argument loop, so it precedes the empty-array
/// `#REF!`, the sparkline-only 0 and the blank-only 0. A `Zoned` mixed with any
/// of those three therefore answers by the zoned rule rather than by theirs —
/// which is exactly the ordering MAX and MIN have always had, and the point of
/// this change is that the four agree.
///
/// [`stat_helpers::zoned_extreme`]: super::stat_helpers::zoned_extreme
///
/// This costs a second recursive pass over the arguments on the common
/// no-zoned path, where it always returns `None`. Accepted deliberately: MAX
/// and MIN already pay it, and paying it is what makes the four answer alike.
pub fn maxa_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // Zone-aware participation, decided identically to MAX — see the doc
    // comment above for why the A-variant does not get its own rule.
    if let Some(r) = super::stat_helpers::zoned_extreme(args, false) {
        return r;
    }
    let mut result: Option<f64> = None;
    // See `fold_array_max` for why this flag exists.
    let mut skipped_sparkline = false;
    let mut saw_date = false;
    for arg in args {
        match arg {
            Value::Sparkline(_) => skipped_sparkline = true,
            Value::Number(n) => {
                result = Some(result.map_or(*n, |cur: f64| cur.max(*n)));
            }
            Value::Date(n) => {
                saw_date = true;
                result = Some(result.map_or(*n, |cur: f64| cur.max(*n)));
            }
            Value::Bool(b) => {
                let n = if *b { 1.0 } else { 0.0 };
                result = Some(result.map_or(n, |cur: f64| cur.max(n)));
            }
            Value::Text(_) => return Value::Error(ErrorKind::Value),
            Value::Empty => {}
            Value::Array(inner) => {
                // An empty argument is #REF!, as it is for MIN and MAX
                // (`=MAXA({})`). Note MAXA reaches the *other* answers by a
                // different route: text folds in as 0 rather than being
                // skipped, so `=MAXA({"a","b"})` is already 0 without any
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
                    fold_array_max(inner, &mut result, &mut skipped_sparkline, &mut saw_date)
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
        // An array of nothing but blanks is 0, not #N/A: `=MAXA(A1:A3)` over
        // empty cells answers the same 0 that MAX, MIN and MINA give it — see
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
/// with MAXA's array-context coercion rules.
/// A sparkline is skipped wherever it appears, and an aggregate whose scope
/// holds nothing else answers 0 — the same answer whether it arrived as a
/// direct argument or through a range (google.tsv: `=MAXA(SPARKLINE({1,2,3}))`
/// and `=MAXA(Data!K1:K1)` are both 0). The flag is what distinguishes "skipped a
/// sparkline" from "saw nothing usable at all", which stay different answers.
fn fold_array_max(
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
                fold_array_max(inner, result, skipped_sparkline, saw_date)?;
                continue;
            }
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            // Listed rather than a catch-all so a new `Value` variant is a
            // compile error here instead of inheriting "skipped" by accident.
            Value::Zoned(_) => continue,
        };
        *result = Some(result.map_or(n, |cur: f64| cur.max(n)));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
