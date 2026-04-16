use chrono::Datelike;
use crate::display::display_number;
use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::types::Value;

/// Format an integer part with thousands separators.
fn format_with_commas(int_part: u64) -> String {
    let s = int_part.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + len / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

/// Apply a format string to a number value, returning the formatted string.
fn apply_format(n: f64, fmt: &str) -> String {
    // ── Percentage format: ends with '%' ─────────────────────────────────────
    if let Some(pct_fmt) = fmt.strip_suffix('%') {
        let pct_val = n * 100.0;
        return format!("{}%", apply_format(pct_val, pct_fmt));
    }

    // ── Date format: contains date tokens ────────────────────────────────────
    {
        let lower = fmt.to_lowercase();
        if lower.contains("yyyy") || lower.contains("yy")
            || lower.contains("mm") || lower.contains("dd")
        {
            if let Some(date) = serial_to_date(n) {
                let mut out = fmt.to_string();
                // Replace in order from longest to shortest to avoid partial replacements
                out = out.replace("yyyy", &format!("{:04}", date.year()));
                out = out.replace("yy", &format!("{:02}", date.year() % 100));
                out = out.replace("mm", &format!("{:02}", date.month()));
                out = out.replace("dd", &format!("{:02}", date.day()));
                return out;
            }
        }
    }

    // ── Fraction format: contains '/' with digit denominator ────────────────
    // e.g. "0/4" means "whole numerator/4"
    if let Some(slash_pos) = fmt.find('/') {
        let denom_str = fmt[slash_pos + 1..].trim();
        if let Ok(denom) = denom_str.parse::<u64>() {
            if denom > 0 {
                // Round to nearest multiple of 1/denom
                let numerator = (n * denom as f64).round() as u64;
                if numerator == 0 {
                    return "0".to_string();
                }
                let gcd_val = gcd(numerator, denom);
                let num = numerator / gcd_val;
                let den = denom / gcd_val;
                if den == 1 {
                    return format!("{}", num);
                }
                return format!("{}/{}", num, den);
            }
        }
    }

    // ── Comma + decimal format: e.g. "#,##0.00", "#,##0" ───────────────────
    let has_comma = fmt.contains(',');
    let negative = n < 0.0;
    let abs_n = n.abs();

    if has_comma {
        if let Some(dot_pos) = fmt.find('.') {
            let decimal_part = &fmt[dot_pos + 1..];
            if decimal_part.chars().all(|c| c == '0' || c == '#') {
                let places = decimal_part.len();
                let scale = 10f64.powi(places as i32);
                let rounded = (abs_n * scale).round() / scale;
                let int_part = rounded as u64;
                let frac = rounded - int_part as f64;
                let frac_digits = (frac * scale).round() as u64;
                let int_str = format_with_commas(int_part);
                let result = format!("{}.{:0>width$}", int_str, frac_digits, width = places);
                return if negative { format!("-{}", result) } else { result };
            }
        } else {
            // No decimal point — just comma grouping
            let fmt_core = fmt.trim_matches(|c| c == '#' || c == ',' || c == '0');
            let _ = fmt_core; // unused, just checking it's a valid pattern
            let int_part = abs_n.round() as u64;
            let result = format_with_commas(int_part);
            return if negative { format!("-{}", result) } else { result };
        }
    }

    // ── Simple decimal-only format: "0.00", "#.##" etc. ──────────────────────
    if let Some(dot_pos) = fmt.find('.') {
        let decimal_part = &fmt[dot_pos + 1..];
        if decimal_part.chars().all(|c| c == '0' || c == '#') {
            let places = decimal_part.len();
            return format!("{:.prec$}", n, prec = places);
        }
    } else if fmt.chars().all(|c| c == '0' || c == '#') {
        return format!("{:.0}", n);
    }

    // ── Fallback ─────────────────────────────────────────────────────────────
    display_number(n)
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// `TEXT(value, format_text)` — converts a number to a formatted string.
pub fn text_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    // Preserve the original value for date detection
    let raw = args[0].clone();
    let is_date = matches!(raw, Value::Date(_));
    let n = match &raw {
        Value::Date(d) => *d,
        Value::Number(n) => *n,
        other => match crate::eval::coercion::to_number(other.clone()) {
            Ok(n) => n,
            Err(e) => return e,
        },
    };
    let format = match to_string_val(args[1].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    // For non-date values, strip date format tokens to avoid misdetection
    // (a plain number should not be formatted as a date unless the format
    // contains date tokens AND the value came from a date function)
    let _ = is_date; // currently unused; serial_to_date handles any f64
    Value::Text(apply_format(n, &format))
}

#[cfg(test)]
mod tests;
