use crate::types::{ErrorKind, Value};

/// `MAX(value1, ...)` — largest numeric value in the arguments.
/// Direct args: Numbers, Dates, Bool (TRUE=1, FALSE=0), parseable text coerced
/// to number.
/// Array elements: Numbers and Dates; text/Bool → skip; errors propagate.
///
/// "No numbers" is *two* rules, not one:
///
/// - an **empty** array argument is `#REF!`: `=MAX({})` — the one rule with
///   an in-repo row (statistical.tsv);
/// - a **populated** array holding nothing numeric is 0: `=MAX({"a","b"})`
///   and `=MAX({TRUE,FALSE})` are both 0, even though neither text nor
///   booleans contribute a number in array context. Captured in Google
///   Sheets; the rows land separately, since they fail until this code exists.
///
/// An array of nothing but *blanks* — what `=MAX(A1:A3)` over an untouched
/// column materializes as — is 0 as well. That is captured across seven range
/// shapes, each with a populated control; the shapes, the controls and where
/// the rows live are set out on [`stat_helpers::is_blank_only_array`], which
/// is what decides the case. Deciding it there rather than through
/// `array_had_content` is deliberate: a blank sitting next to something else
/// numberless changes nothing.
///
/// `array_had_content` is still set by text and booleans alone, so it exempts
/// exactly what those captures cover and makes no claim past them. A zoned
/// instant is the remaining numberless case: unprobed, and still `#REF!`.
///
/// [`stat_helpers::is_blank_only_array`]: super::stat_helpers::is_blank_only_array
///
/// **Dates participate as bare serials** and carry their type out. A date-only
/// range answers the latest date, a date beside a plain number is compared on
/// the serial with no special casing, and the result is date-typed whenever a
/// date took part — even when a plain number won the comparison.
///
/// Captured and extrapolated are not the same thing here, so keep them apart:
///
/// - **Values** are captured in Google Sheets for every form — a date-only
///   column, a date/number column, array literals of both shapes, and dates
///   passed as direct arguments.
/// - **Typing** is captured for the *range* forms only, read back through the
///   cell that holds the result (`=MAX(<date-only range>)` and
///   `=MAX(<date/number range>)` both come back `date`). The literal and
///   direct-argument rows report `number`, but that is an artifact of the
///   capture harness reading them through an `INDEX(...,1,1)` wrapper, which
///   drops the cell's date format — it is not a Sheets answer. So the date
///   typing of `=MAX({DATE(...),DATE(...)})` is **extrapolated** from the
///   range forms, not probed.
///
/// **None of those rows are in this repo yet.** They come off the
/// conformance-fixtures pipeline and land in a separate fixtures-only PR —
/// they fail until this code exists, and CI rejects a PR that mixes fixture
/// TSVs with code. Same arrangement as the blank-only rows described on
/// `stat_helpers::is_blank_only_array`. A reviewer working from this repo
/// alone can check the unit tests and this comment; the Sheets answers
/// themselves have to be taken from that pipeline.
///
/// Two consequences were filed separately and are now closed:
///
/// - `COUNT` did not count dates, so `=COUNT(MAX(<date range>))` answered 0
///   where it used to answer 1. Fixed in `count` against captured Sheets rows
///   (#780).
/// - `MAXA`/`MINA` silently dropped a `Zoned` sitting beside a `Date`, where
///   `MAX`/`MIN` route the same input through `zoned_extreme` and error. The
///   A-variants now consult the same helper, so all four agree; the rule is a
///   deliberate truecalc-only decision recorded on `maxa_fn` (#781).
pub fn max_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // Zone-aware participation: a column of Zoned instants returns the latest
    // (as a Zoned); mixing naive and aware values is #VALUE!.
    if let Some(r) = super::stat_helpers::zoned_extreme(args, false) {
        return r;
    }
    let mut result: Option<f64> = None;
    let mut had_array = false;
    let mut array_had_content = false;
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
            Value::Text(s) => {
                let trimmed = s.trim();
                match trimmed.parse::<f64>() {
                    Ok(v) if v.is_finite() => {
                        result = Some(result.map_or(v, |cur: f64| cur.max(v)));
                    }
                    _ => return Value::Error(ErrorKind::Value),
                }
            }
            Value::Empty => {}
            Value::Array(elems) => {
                had_array = true;
                if elems.is_empty() {
                    return Value::Error(ErrorKind::Ref);
                }
                // Recurse into nested arrays (e.g. a vertical range
                // materializes as nested one-element row arrays) so every
                // cell is visited.
                if let Err(e) = max_array_into(
                    elems,
                    &mut result,
                    &mut skipped_sparkline,
                    &mut array_had_content,
                    &mut saw_date,
                ) {
                    return e;
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
            // Listed rather than a catch-all so a new `Value` variant is a
            // compile error here instead of a silent skip. A `Zoned` only
            // reaches this loop when no other argument was zone-aware, which
            // `zoned_extreme` above has already ruled on.
            Value::Zoned(_) => {}
        }
    }
    // A skipped sparkline is not "nothing usable": the aggregate had something
    // in scope, so it answers 0 rather than falling into the rule below
    // (google.tsv: `=MAX(Data!K1:K1)` is 0). This runs first, so it decides
    // every sparkline case before `array_had_content` is consulted at all.
    if skipped_sparkline && result.is_none() {
        return Value::Number(0.0);
    }
    // An array holding text or booleans answers 0 (`=MAX({"a","b"})` and
    // `=MAX({TRUE,FALSE})` are both 0). `array_had_content` is set by exactly
    // those two variants and nothing else, so it exempts what that capture
    // covers and makes no claim beyond it. Dates never reach here: they
    // contribute a number, so `result` is `Some` whenever one was seen.
    if had_array && !array_had_content && result.is_none() {
        // An array of nothing but blanks is a further exemption, and it is
        // captured: `=MAX(A1:A3)` over empty cells is 0, the same answer MIN,
        // MAXA and MINA give it — see `is_blank_only_array` for every range
        // shape that was probed and where the rows live. The check is on the
        // arguments as a whole, so a blank mixed with anything else is
        // untouched by this rule.
        if super::stat_helpers::is_blank_only_array(args) {
            return Value::Number(0.0);
        }
        return Value::Error(ErrorKind::Ref);
    }
    match result {
        // A date anywhere in scope makes the answer date-typed, whether or not
        // the date is the value that won.
        Some(n) if saw_date => Value::Date(n),
        Some(n) => Value::Number(n),
        None => Value::Number(0.0),
    }
}

/// Recursively fold a nested array's numbers into `result` for MAX's
/// array-context rules (Bool/Text/Empty skipped, errors propagate).
/// A sparkline is skipped wherever it appears, and an aggregate whose scope
/// holds nothing else answers 0 — the same answer whether it arrived as a
/// direct argument or through a range (google.tsv: `=MAX(SPARKLINE({1,2,3}))`
/// and `=MAX(Data!K1:K1)` are both 0). The flag is what distinguishes "skipped a
/// sparkline" from "saw nothing usable at all", which stay different answers.
///
/// A `Date` folds in as its bare serial and raises `saw_date`, which types the
/// answer; it never touches `had_content`, since a date always leaves a number
/// behind and so can never reach the numberless rule.
///
/// `had_content` is set by text and booleans *only* — the two variants the
/// capture covers (`=MAX({"a","b"})` and `=MAX({TRUE,FALSE})` are both 0).
/// It is deliberately not a catch-all: every other non-numeric variant leaves
/// it alone and so keeps the `#REF!` MAX has always answered. Listing the
/// variants rather than falling through also stops a future `Value` kind
/// inheriting content-hood by accident. Two variants no longer end at `#REF!`,
/// and neither gets there through this flag: an all-blank array is decided by
/// the blank-only check in `max_fn`, and a `Date` contributes a number so the
/// numberless rule is never reached at all.
fn max_array_into(
    elems: &[Value],
    result: &mut Option<f64>,
    skipped_sparkline: &mut bool,
    had_content: &mut bool,
    saw_date: &mut bool,
) -> Result<(), Value> {
    for elem in elems {
        match elem {
            Value::Number(n) => {
                *result = Some(result.map_or(*n, |cur: f64| cur.max(*n)));
            }
            Value::Date(n) => {
                *saw_date = true;
                *result = Some(result.map_or(*n, |cur: f64| cur.max(*n)));
            }
            Value::Text(_) | Value::Bool(_) => *had_content = true,
            Value::Sparkline(_) => *skipped_sparkline = true,
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            Value::Array(inner) => {
                max_array_into(inner, result, skipped_sparkline, had_content, saw_date)?
            }
            // Listed rather than a catch-all so a new `Value` variant is a
            // compile error here instead of inheriting "skipped, and not
            // content either" by accident.
            Value::Empty | Value::Zoned(_) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
