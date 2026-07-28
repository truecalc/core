use chrono::Datelike;
use crate::display::display_number;
use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::serial_to_date;
use crate::types::Value;

/// Apply a format string to a number value, returning the formatted string.
fn apply_format(n: f64, fmt: &str) -> String {
    // GS: asterisk fill formats (*<char>) are not supported; return "0"
    if fmt.starts_with('*') {
        return "0".to_string();
    }

    // ── Percentage format: ends with '%' ─────────────────────────────────────
    if let Some(pct_fmt) = fmt.strip_suffix('%') {
        let pct_val = n * 100.0;
        return format!("{}%", apply_format(pct_val, pct_fmt));
    }

    // ── Time format: contains time tokens (h, hh, ss, AM/PM) ─────────────────
    {
        let lower = fmt.to_lowercase();
        let is_time_fmt = lower.contains("hh") || lower.contains("ss")
            || lower.contains("am/pm") || lower.contains("a/p")
            || (lower.contains('h') && !lower.contains("yyyy") && !lower.contains("yy"));
        if is_time_fmt {
            // n is a fraction of a day; 0.5 = noon
            let total_secs = (n.fract().abs() * 86400.0).round() as u64;
            let hours = total_secs / 3600;
            let minutes = (total_secs % 3600) / 60;
            let seconds = total_secs % 60;

            let use_ampm = lower.contains("am/pm") || lower.contains("a/p");
            let (display_hours, ampm_str) = if use_ampm {
                let h12 = if hours == 0 { 12 } else if hours > 12 { hours - 12 } else { hours };
                let ap = if hours < 12 { "AM" } else { "PM" };
                (h12, ap)
            } else {
                (hours, "")
            };

            // Replace tokens in order: longest first to avoid partial replacements
            let mut out = fmt.to_string();
            // Remove AM/PM or A/P token first
            out = out.replace("AM/PM", ampm_str);
            out = out.replace("am/pm", ampm_str);
            out = out.replace("A/P", ampm_str);
            out = out.replace("a/p", ampm_str);
            // Replace hh before h
            out = out.replace("hh", &format!("{:02}", display_hours));
            out = out.replace('h', &display_hours.to_string());
            // Replace ss
            out = out.replace("ss", &format!("{:02}", seconds));
            // Replace mm (minutes in time context)
            out = out.replace("mm", &format!("{:02}", minutes));
            return out.trim().to_string();
        }
    }

    // ── Date format: contains date tokens ────────────────────────────────────
    {
        let lower = fmt.to_lowercase();
        if lower.contains("yyyy") || lower.contains("yy")
            || lower.contains("mm") || lower.contains("dd")
        {
            if let Some(date) = serial_to_date(n) {
                // Work on a lowercase copy so replacements are case-insensitive,
                // then return the normalised (lowercase) result.
                let mut out = lower;
                // Replace in order from longest to shortest to avoid partial replacements
                out = out.replace("yyyy", &format!("{:04}", date.year()));
                out = out.replace("yy", &format!("{:02}", date.year() % 100));
                out = out.replace("mm", &format!("{:02}", date.month()));
                out = out.replace("dd", &format!("{:02}", date.day()));
                return out;
            }
        }
    }

    // ── Scientific notation: e.g. "0.00E+00" ────────────────────────────────
    // Detect only when format has digit tokens before E and sign+digits after.
    {
        // Find 'E' or 'e' that is preceded by a digit token and followed by '+'/'-'
        let sci_pos = fmt.char_indices().find(|&(i, c)| {
            if c != 'E' && c != 'e' {
                return false;
            }
            let before = &fmt[..i];
            let after = &fmt[i + 1..];
            // before must contain at least one '0' or '#'
            let has_digit_token = before.contains('0') || before.contains('#');
            // after must start with '+' or '-'
            let has_sign = after.starts_with('+') || after.starts_with('-');
            has_digit_token && has_sign
        });

        if let Some((e_pos, _)) = sci_pos {
            // Count decimal places before E (e.g. "0.00E+00" → 2)
            let before_e = &fmt[..e_pos];
            let decimal_places = if let Some(dot_pos) = before_e.find('.') {
                before_e[dot_pos + 1..].len()
            } else {
                0
            };
            // Count exponent digits after E+/- sign
            let after_e = &fmt[e_pos + 1..];
            let exp_digits = after_e.trim_start_matches(['+', '-']).len();
            let exp_digits = exp_digits.max(2);

            // Compute mantissa and exponent
            let abs_n = n.abs();
            let (mantissa, exponent) = if abs_n == 0.0 {
                (0.0_f64, 0_i32)
            } else {
                let exp = libm::log10(abs_n).floor() as i32;
                (abs_n / 10f64.powi(exp), exp)
            };
            let sign = if n < 0.0 { "-" } else { "" };
            let exp_sign = if exponent < 0 { "-" } else { "+" };
            return format!(
                "{}{:.prec$}E{}{:0>width$}",
                sign,
                mantissa,
                exp_sign,
                exponent.unsigned_abs(),
                prec = decimal_places,
                width = exp_digits,
            );
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

    // ── Number format: handles prefix, commas, #/0/? digit tokens ─────────────
    // Extract optional leading prefix (anything before the first digit token or comma)
    let digit_token = |c: char| c == '#' || c == '0' || c == '?';
    let prefix_end = fmt.char_indices()
        .find(|&(_, c)| digit_token(c) || c == ',')
        .map(|(i, _)| i)
        .unwrap_or(fmt.len());
    let prefix = &fmt[..prefix_end];
    let rest = &fmt[prefix_end..];

    let has_comma = rest.contains(',');
    let negative = n < 0.0;
    let abs_n = n.abs();

    // Split rest into integer and fractional format parts
    let (int_fmt, frac_fmt) = if let Some(dot_pos) = rest.find('.') {
        (&rest[..dot_pos], &rest[dot_pos + 1..])
    } else {
        (rest, "")
    };

    // Strip commas from int_fmt to get the actual digit pattern
    let int_pattern: String = int_fmt.chars().filter(|c| *c != ',').collect();

    // Only proceed if all remaining chars are digit tokens
    let valid_int = int_pattern.chars().all(|c| c == '#' || c == '0' || c == '?');
    let valid_frac = frac_fmt.chars().all(|c| c == '#' || c == '0' || c == '?');

    // Only apply the digit-format branch when there's at least one actual digit token
    let has_any_digit_token = int_pattern.contains(['#', '0', '?'])
        || frac_fmt.contains(['#', '0', '?']);

    if valid_int && valid_frac && has_any_digit_token {
        // Count minimum integer digits (number of '0' tokens)
        let min_int_digits = int_pattern.chars().filter(|&c| c == '0').count().max(1);
        let total_int_tokens = int_pattern.len();

        // Count fractional tokens
        let frac_len = frac_fmt.len();
        let frac_places_needed = frac_fmt.chars().filter(|&c| c == '0').count();
        // Max decimal places = all tokens; trailing # suppress trailing zeros
        let max_frac_places = frac_len;

        // Round to max_frac_places
        let scale = 10f64.powi(max_frac_places as i32);
        let rounded = (abs_n * scale).round() / scale;
        let int_part = rounded.trunc() as u64;
        let frac_val = rounded - int_part as f64;

        // Format integer part with minimum digits
        let int_str_raw = format!("{:0>width$}", int_part, width = min_int_digits);
        // Suppress leading chars beyond total tokens if all are '#'
        // (GS: leading # means suppress if zero)
        let int_str = if total_int_tokens > min_int_digits && int_str_raw.len() < total_int_tokens {
            int_str_raw.clone()
        } else {
            int_str_raw
        };

        // Apply comma grouping if needed
        let int_formatted = if has_comma {
            // insert commas in int_str
            let s = &int_str;
            let len = s.len();
            let mut r = String::with_capacity(len + len / 3);
            for (i, c) in s.chars().enumerate() {
                if i > 0 && (len - i).is_multiple_of(3) {
                    r.push(',');
                }
                r.push(c);
            }
            r
        } else {
            int_str.clone()
        };

        // Format fractional part
        let frac_str = if max_frac_places == 0 {
            String::new()
        } else {
            let frac_digits = (frac_val * scale).round() as u64;
            let full = format!("{:0>width$}", frac_digits, width = max_frac_places);
            // Apply per-token rules: # = suppress trailing zeros, ? = space-pad, 0 = keep
            let chars: Vec<char> = full.chars().collect();
            let tokens: Vec<char> = frac_fmt.chars().collect();
            // Find last non-suppressible position (# suppresses trailing zeros)
            let keep_up_to = {
                let mut last_keep = 0usize;
                for (i, &tok) in tokens.iter().enumerate() {
                    if tok == '0' || tok == '?' {
                        last_keep = i + 1; // must include at least up to here
                    } else if tok == '#' && chars[i] != '0' {
                        last_keep = i + 1; // # includes if non-zero
                    }
                }
                last_keep
            };
            let mut frac_out = String::new();
            for (i, &tok) in tokens.iter().enumerate() {
                if i < keep_up_to {
                    if tok == '?' && chars[i] == '0' && i >= frac_places_needed {
                        frac_out.push(' '); // ? pads with space instead of zero when trailing
                    } else {
                        frac_out.push(chars[i]);
                    }
                } else if tok == '?' {
                    frac_out.push(' ');
                }
                // '#' beyond keep_up_to: omit
            }
            frac_out
        };

        let number_str = if frac_str.is_empty() {
            format!("{}{}", prefix, int_formatted)
        } else {
            format!("{}{}.{}", prefix, int_formatted, frac_str)
        };

        return if negative { format!("-{}", number_str) } else { number_str };
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
    // GS: boolean input is NOT coerced to a number; it is returned as "TRUE" or "FALSE".
    if let Value::Bool(b) = raw {
        return Value::Text(if b { "TRUE".to_string() } else { "FALSE".to_string() });
    }
    // google.tsv: `=TEXT(SPARKLINE({1,2,3}),"0")` is the empty string, so TEXT
    // needs its own answer — it reads its value as a number, which a sparkline
    // is not. Note this is *not* "as if empty text" either: `TEXT("","0")`
    // formats as "0".
    //
    // There is no rule to derive this from: the whole `TO_*` family also answers
    // "" while `DOLLAR` and `FIXED` answer `#VALUE!` — `DOLLAR` and
    // `TO_DOLLARS` differ despite the names. The set of exceptions is
    // empirical: see the coercion map in `crate::eval::functions::google` for
    // which functions were probed in which direction, and extend that list from
    // the oracle rather than by analogy.
    if matches!(raw, Value::Sparkline(_)) {
        return Value::Text(String::new());
    }
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
