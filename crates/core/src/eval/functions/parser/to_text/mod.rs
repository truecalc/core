use crate::display::display_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// Format a number as Google Sheets `TO_TEXT` would.
///
/// - For `abs(n) >= 1e15` Sheets switches to upper-case scientific notation
///   with an explicit sign on the exponent: `"1E+15"`, `"-2.5E+20"`.
/// - For everything else, use the standard spreadsheet display formatter
///   (`display_number`), which strips trailing zeros and gives up to 14 sig
///   figs — matching what Sheets shows in the formula bar.
fn gs_number_to_text(n: f64) -> String {
    if n.is_nan() || n.is_infinite() {
        return "#NUM!".to_string();
    }
    let abs = n.abs();
    if abs >= 1e15 {
        // Rust's {:E} gives e.g. "1E15" — we need "1E+15" (explicit + sign).
        let raw = format!("{:E}", n);
        // Insert '+' after 'E' when the exponent has no sign.
        if let Some(pos) = raw.find('E') {
            let after = &raw[pos + 1..];
            if !after.starts_with('-') && !after.starts_with('+') {
                return format!("{}E+{}", &raw[..pos], after);
            }
        }
        raw
    } else {
        display_number(n)
    }
}

pub fn to_text_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match &args[0] {
        Value::Number(n) => Value::Text(gs_number_to_text(*n)),
        Value::Date(n)   => Value::Text(gs_number_to_text(*n)),
        Value::Bool(b)   => Value::Text(if *b { "TRUE".to_string() } else { "FALSE".to_string() }),
        Value::Text(s)   => Value::Text(s.clone()),
        Value::Error(_) | Value::ErrorMsg(_, _)  => args[0].clone(),
        // Canonical statement for the whole `TO_*` family (`TO_TEXT`,
        // `TO_PERCENT`, `TO_DOLLARS`, `TO_PURE_NUMBER`, `TO_DATE`): every member
        // reads a sparkline as the empty string (google.tsv). The family is
        // uniform, but nothing *outside* it can be inferred from that —
        // `DOLLAR` rejects a sparkline while `TO_DOLLARS` does not. The other
        // members point here rather than restating it; see also the coercion
        // section of `crate::eval::functions::google`.
        Value::Sparkline(_) => Value::Text(String::new()),
        _                => Value::Error(ErrorKind::Value),
    }
}

#[cfg(test)]
mod tests;
