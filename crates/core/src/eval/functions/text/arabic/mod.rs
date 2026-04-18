use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `ARABIC(roman_text)` — converts a Roman numeral string to an integer.
/// Returns 0 for empty string. Returns `#VALUE!` for invalid Roman numerals.
pub fn arabic_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    if text.is_empty() {
        return Value::Number(0.0);
    }
    match parse_roman(&text.to_uppercase()) {
        Some(n) => Value::Number(n as f64),
        None => Value::Error(ErrorKind::Value),
    }
}

fn roman_value(c: char) -> Option<i32> {
    match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    }
}

fn parse_roman(s: &str) -> Option<i32> {
    let chars: Vec<char> = s.chars().collect();
    // Validate all chars are valid Roman numerals first
    for &c in &chars {
        roman_value(c)?;
    }
    let mut total = 0i32;
    let mut prev = 0i32;
    for &c in chars.iter().rev() {
        let val = roman_value(c)?;
        if val < prev {
            total -= val;
        } else {
            total += val;
        }
        prev = val;
    }
    Some(total)
}

#[cfg(test)]
mod tests;
