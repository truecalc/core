use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// Convert a full-width Unicode character to its half-width equivalent.
/// U+FF01–U+FF5E → U+0021–U+007E (full-width ASCII variants)
/// U+3000 → U+0020 (ideographic space → ASCII space)
fn to_halfwidth(c: char) -> char {
    let cp = c as u32;
    if cp == 0x3000 {
        ' '
    } else if (0xFF01..=0xFF5E).contains(&cp) {
        char::from_u32(cp - 0xFEE0).unwrap_or(c)
    } else {
        c
    }
}

/// `ASC(text)` — converts full-width characters to half-width equivalents.
pub fn asc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    Value::Text(text.chars().map(to_halfwidth).collect())
}

#[cfg(test)]
mod tests;
