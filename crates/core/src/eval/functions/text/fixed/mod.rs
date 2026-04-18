use crate::eval::coercion::{to_bool, to_number};
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `FIXED(number, [decimals=2], [no_commas=FALSE])` — formats a number as text
/// with a fixed number of decimal places, optionally with thousands separators.
pub fn fixed_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 3) {
        return err;
    }
    let number = match to_number(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let decimals = if args.len() >= 2 {
        match to_number(args[1].clone()) {
            Ok(v) => v as i32,
            Err(e) => return e,
        }
    } else {
        2
    };
    let no_commas = if args.len() >= 3 {
        match to_bool(args[2].clone()) {
            Ok(v) => v,
            Err(e) => return e,
        }
    } else {
        false
    };

    Value::Text(format_fixed(number, decimals, no_commas))
}

pub(crate) fn format_fixed(number: f64, decimals: i32, no_commas: bool) -> String {
    let negative = number < 0.0;
    let abs_val = number.abs();

    // Round to the requested decimal places (or to tens/hundreds if decimals < 0)
    let scale = 10f64.powi(decimals);
    let rounded = (abs_val * scale).round() / scale;

    let formatted = if decimals <= 0 {
        // No decimal portion; optionally round to left of decimal
        // Scale back: e.g. decimals=-1 means rounded to nearest 10
        let final_int = if decimals < 0 {
            let factor = 10u64.pow((-decimals) as u32);
            ((abs_val / factor as f64).round() as u64) * factor
        } else {
            rounded as u64
        };
        if no_commas {
            format!("{}", final_int)
        } else {
            format_with_commas_int(final_int)
        }
    } else {
        // Has decimal portion
        let int_part = rounded as u64;
        // Compute decimal digits carefully
        let frac = rounded - int_part as f64;
        let frac_digits = (frac * scale).round() as u64;
        let int_str = if no_commas {
            format!("{}", int_part)
        } else {
            format_with_commas_int(int_part)
        };
        format!("{}.{:0>width$}", int_str, frac_digits, width = decimals as usize)
    };

    if negative {
        format!("-{}", formatted)
    } else {
        formatted
    }
}

fn format_with_commas_int(n: u64) -> String {
    let s = n.to_string();
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

#[cfg(test)]
mod tests;
