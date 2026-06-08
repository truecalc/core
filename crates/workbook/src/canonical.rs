//! RFC 8785 (JSON Canonicalization Scheme, JCS) serializer (schema spec §8).
//!
//! Stock `serde_json` is **not** JCS-conformant for numbers: JCS requires
//! ECMAScript `Number::toString` notation, which switches to exponent form at
//! `1e+21` and at `1e-7`, where `serde_json` emits plain decimal expansions
//! (schema spec §8, normative implementation warning). This module therefore
//! formats every number with [`ryu_js`] (ECMAScript-conformant shortest
//! round-trip) and emits object keys sorted by ascending UTF-16 code units.
//!
//! Adopting JCS wholesale means any off-the-shelf JCS implementation can
//! independently verify TrueCalc's canonical bytes — third parties can audit
//! the determinism claim without trusting our code.
//!
//! The input is a [`serde_json::Value`] tree produced from the typed workbook
//! by `serde_json::to_value`. The domain orderings of schema spec §8.7
//! (`sheets` keeps tab order; `names` sorted by `name`; arrays keep row-major
//! order) are imposed *before* canonicalization, by the producing code; pure
//! JCS key ordering then applies uniformly to every object (so `A10` sorts
//! before `A2` inside a `cells` map — schema spec §8.1).

use serde_json::Value;

/// Serializes a JSON tree to RFC 8785 canonical bytes: no insignificant
/// whitespace, object keys sorted by UTF-16 code units, JCS number formatting,
/// JCS minimal string escaping. The result has no trailing newline (schema
/// spec §1).
///
/// Returns an error if any number is non-finite — JCS and this schema forbid
/// NaN and Infinity (schema spec §8.4). (The value layer already rejects them,
/// so this is a belt-and-braces invariant guard.)
pub fn to_canonical_string(value: &Value) -> Result<String, String> {
    let mut out = String::new();
    write_value(value, &mut out)?;
    Ok(out)
}

fn write_value(value: &Value, out: &mut String) -> Result<(), String> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => write_number(n, out)?,
        Value::String(s) => write_string(s, out),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(item, out)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            // JCS: sort keys by ascending UTF-16 code units (schema spec §8.1).
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| cmp_utf16(a, b));
            out.push('{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push(':');
                write_value(&map[*key], out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

/// JCS number formatting: ECMAScript `Number::toString` of the f64 value
/// (schema spec §8.2). Integral values print without a fraction (`8`, not
/// `8.0`); `0.1 + 0.2` prints as `0.30000000000000004`; the `1e21` and `1e-7`
/// exponent boundaries match ECMAScript exactly. `-0` serializes as `0`
/// (schema spec §8.3).
fn write_number(n: &serde_json::Number, out: &mut String) -> Result<(), String> {
    let f = n
        .as_f64()
        .ok_or_else(|| format!("number {n} is not representable as f64"))?;
    if !f.is_finite() {
        return Err("non-finite numbers are forbidden by JCS (schema spec §8.4)".to_string());
    }
    let f = if f == 0.0 { 0.0 } else { f }; // normalize -0 to 0 (schema spec §8.3)
    let mut buf = ryu_js::Buffer::new();
    out.push_str(buf.format(f));
    Ok(())
}

/// JCS string escaping (schema spec §8.5): escape only the characters RFC 8785
/// requires — `"`, `\`, and C0 controls (with the short forms `\b \t \n \f \r`,
/// `\uXXXX` otherwise). Every other code point, including all non-ASCII, is
/// emitted as literal UTF-8.
fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{09}' => out.push_str("\\t"),
            '\u{0A}' => out.push_str("\\n"),
            '\u{0C}' => out.push_str("\\f"),
            '\u{0D}' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Compares two strings by ascending UTF-16 code-unit sequence, per JCS
/// (schema spec §8.1). For the ASCII keys this schema defines this is identical
/// to byte order, but the full UTF-16 rule is implemented so the canonical form
/// is verifiable by off-the-shelf JCS tooling for any key.
fn cmp_utf16(a: &str, b: &str) -> std::cmp::Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}
