use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `DOLLAR(number, [decimals=2])` — formats a number as currency text (e.g. "$1,234.57").
/// Negative numbers format as "-$1,234.57". Negative decimals rounds to the left of decimal.
pub fn dollar_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
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

    Value::Text(format_dollar(number, decimals))
}

fn format_dollar(number: f64, decimals: i32) -> String {
    let negative = number < 0.0;
    let abs_val = number.abs();

    let body = if decimals < 0 {
        // Round to left of decimal point
        let factor = 10f64.powi(-decimals);
        let rounded = (abs_val / factor).round() as u64 * factor as u64;
        format_with_commas_int(rounded)
    } else {
        // Round to given decimal places
        let scale = 10f64.powi(decimals);
        let rounded = (abs_val * scale).round() / scale;
        let int_part = rounded as u64;
        let int_str = format_with_commas_int(int_part);
        if decimals == 0 {
            int_str
        } else {
            let frac = rounded - int_part as f64;
            let frac_digits = (frac * scale).round() as u64;
            format!("{}.{:0>width$}", int_str, frac_digits, width = decimals as usize)
        }
    };

    if negative {
        format!("-${}", body)
    } else {
        format!("${}", body)
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
