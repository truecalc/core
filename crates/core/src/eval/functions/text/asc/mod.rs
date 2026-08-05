use crate::eval::coercion::to_string_val;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// Full-width katakana / punctuation to half-width mapping.
fn kata_to_halfwidth(cp: u32) -> Option<&'static [u32]> {
    match cp {
        0x30A1 => Some(&[0xFF67]),
        0x30A2 => Some(&[0xFF71]),
        0x30A3 => Some(&[0xFF68]),
        0x30A4 => Some(&[0xFF72]),
        0x30A5 => Some(&[0xFF69]),
        0x30A6 => Some(&[0xFF73]),
        0x30A7 => Some(&[0xFF6A]),
        0x30A8 => Some(&[0xFF74]),
        0x30A9 => Some(&[0xFF6B]),
        0x30AA => Some(&[0xFF75]),
        0x30AB => Some(&[0xFF76]),
        0x30AC => Some(&[0xFF76, 0xFF9E]),
        0x30AD => Some(&[0xFF77]),
        0x30AE => Some(&[0xFF77, 0xFF9E]),
        0x30AF => Some(&[0xFF78]),
        0x30B0 => Some(&[0xFF78, 0xFF9E]),
        0x30B1 => Some(&[0xFF79]),
        0x30B2 => Some(&[0xFF79, 0xFF9E]),
        0x30B3 => Some(&[0xFF7A]),
        0x30B4 => Some(&[0xFF7A, 0xFF9E]),
        0x30B5 => Some(&[0xFF7B]),
        0x30B6 => Some(&[0xFF7B, 0xFF9E]),
        0x30B7 => Some(&[0xFF7C]),
        0x30B8 => Some(&[0xFF7C, 0xFF9E]),
        0x30B9 => Some(&[0xFF7D]),
        0x30BA => Some(&[0xFF7D, 0xFF9E]),
        0x30BB => Some(&[0xFF7E]),
        0x30BC => Some(&[0xFF7E, 0xFF9E]),
        0x30BD => Some(&[0xFF7F]),
        0x30BE => Some(&[0xFF7F, 0xFF9E]),
        0x30BF => Some(&[0xFF80]),
        0x30C0 => Some(&[0xFF80, 0xFF9E]),
        0x30C1 => Some(&[0xFF81]),
        0x30C2 => Some(&[0xFF81, 0xFF9E]),
        0x30C3 => Some(&[0xFF6F]),
        0x30C4 => Some(&[0xFF82]),
        0x30C5 => Some(&[0xFF82, 0xFF9E]),
        0x30C6 => Some(&[0xFF83]),
        0x30C7 => Some(&[0xFF83, 0xFF9E]),
        0x30C8 => Some(&[0xFF84]),
        0x30C9 => Some(&[0xFF84, 0xFF9E]),
        0x30CA => Some(&[0xFF85]),
        0x30CB => Some(&[0xFF86]),
        0x30CC => Some(&[0xFF87]),
        0x30CD => Some(&[0xFF88]),
        0x30CE => Some(&[0xFF89]),
        0x30CF => Some(&[0xFF8A]),
        0x30D0 => Some(&[0xFF8A, 0xFF9E]),
        0x30D1 => Some(&[0xFF8A, 0xFF9F]),
        0x30D2 => Some(&[0xFF8B]),
        0x30D3 => Some(&[0xFF8B, 0xFF9E]),
        0x30D4 => Some(&[0xFF8B, 0xFF9F]),
        0x30D5 => Some(&[0xFF8C]),
        0x30D6 => Some(&[0xFF8C, 0xFF9E]),
        0x30D7 => Some(&[0xFF8C, 0xFF9F]),
        0x30D8 => Some(&[0xFF8D]),
        0x30D9 => Some(&[0xFF8D, 0xFF9E]),
        0x30DA => Some(&[0xFF8D, 0xFF9F]),
        0x30DB => Some(&[0xFF8E]),
        0x30DC => Some(&[0xFF8E, 0xFF9E]),
        0x30DD => Some(&[0xFF8E, 0xFF9F]),
        0x30DE => Some(&[0xFF8F]),
        0x30DF => Some(&[0xFF90]),
        0x30E0 => Some(&[0xFF91]),
        0x30E1 => Some(&[0xFF92]),
        0x30E2 => Some(&[0xFF93]),
        0x30E3 => Some(&[0xFF6C]),
        0x30E4 => Some(&[0xFF94]),
        0x30E5 => Some(&[0xFF6D]),
        0x30E6 => Some(&[0xFF95]),
        0x30E7 => Some(&[0xFF6E]),
        0x30E8 => Some(&[0xFF96]),
        0x30E9 => Some(&[0xFF97]),
        0x30EA => Some(&[0xFF98]),
        0x30EB => Some(&[0xFF99]),
        0x30EC => Some(&[0xFF9A]),
        0x30ED => Some(&[0xFF9B]),
        0x30EF => Some(&[0xFF9C]),
        0x30F0 => Some(&[0xFF72]),
        0x30F1 => Some(&[0xFF74]),
        0x30F2 => Some(&[0xFF66]),
        0x30F3 => Some(&[0xFF9D]),
        0x30F4 => Some(&[0xFF73, 0xFF9E]),
        0x30F5 => Some(&[0xFF76]),
        0x30F6 => Some(&[0xFF79]),
        0x30FB => Some(&[0xFF65]),
        0x30FC => Some(&[0xFF70]),
        0x3001 => Some(&[0xFF64]),
        0x3002 => Some(&[0xFF61]),
        0x300C => Some(&[0xFF62]),
        0x300D => Some(&[0xFF63]),
        0x3000 => Some(&[0x0020]),
        _ => None,
    }
}

/// `ASC(text)` — converts full-width characters to half-width equivalents.
/// Full-width ASCII variants (U+FF01-U+FF5E) map to ASCII counterparts.
/// Full-width katakana maps to half-width katakana (voiced kana decompose into
/// base + combining voiced/semi-voiced mark).
pub fn asc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let text = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let mut result = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        let cp = c as u32;
        if let Some(halfs) = kata_to_halfwidth(cp) {
            for &h in halfs {
                if let Some(hc) = char::from_u32(h) {
                    result.push(hc);
                }
            }
        } else if (0xFF01..=0xFF5E).contains(&cp) {
            if let Some(hc) = char::from_u32(cp - 0xFEE0) {
                result.push(hc);
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    Value::Text(result)
}

#[cfg(test)]
mod tests;
