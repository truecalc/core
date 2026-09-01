//! Named-range `name` and `ref` validity (schema spec §7, follow-up #568 rule 4).
//!
//! A `name` must match `^[A-Za-z_][A-Za-z0-9_]*$`, must not parse as an A1
//! address or an R1C1-style reference, and must not be a boolean literal.
//! A `ref` must already be in canonical sheet-qualified form
//! (`Sheet!A1` / `Sheet!A1:B2`): top-left-first endpoints, single-cell collapse
//! of degenerate ranges, sheet name single-quoted **iff** it is not a bare
//! identifier or would parse as an A1 address (embedded `'` doubled). Because
//! the contract is strict, `from_json` *rejects* a non-canonical `ref` rather
//! than silently rewriting it — that keeps `to_json ∘ from_json = id`.

use crate::address::parse_a1;

/// Returns true if `name` is a syntactically valid named-range name
/// (schema spec §7): `^[A-Za-z_][A-Za-z0-9_]*$`, not an A1 address, not an
/// R1C1-style reference, not a boolean literal.
pub fn is_valid_name(name: &str) -> bool {
    if !is_bare_identifier(name) {
        return false;
    }
    if looks_like_a1(name) || looks_like_r1c1(name) || is_boolean_literal(name) {
        return false;
    }
    true
}

/// `^[A-Za-z_][A-Za-z0-9_]*$`.
pub fn is_bare_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Would `s` parse as a plain A1 address (e.g. `A1`, `BC42`)? Reuses the same
/// bounds-checked grammar as cell keys (schema spec §3).
fn looks_like_a1(s: &str) -> bool {
    parse_a1(s).is_some()
}

/// R1C1-style reference pattern, e.g. `R1C1`, `R2C5` — `R` digits `C` digits,
/// case-insensitive (Sheets' exact exclusion rule is fixture-verified; this is
/// the conservative pattern of schema spec §7).
fn looks_like_r1c1(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() || !bytes[0].eq_ignore_ascii_case(&b'R') {
        return false;
    }
    let rest = &bytes[1..];
    let c_pos = rest.iter().position(|b| b.eq_ignore_ascii_case(&b'C'));
    let Some(c_pos) = c_pos else { return false };
    let row_digits = &rest[..c_pos];
    let col_digits = &rest[c_pos + 1..];
    !row_digits.is_empty()
        && !col_digits.is_empty()
        && row_digits.iter().all(u8::is_ascii_digit)
        && col_digits.iter().all(u8::is_ascii_digit)
}

fn is_boolean_literal(s: &str) -> bool {
    s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("false")
}

/// A parsed named-range reference.
pub struct ParsedRef {
    /// The unquoted sheet name the ref targets.
    pub sheet: String,
}

/// Parses a `ref` and verifies it is in canonical form (schema spec §7).
/// Returns the targeted (unquoted) sheet name on success, or an error string
/// describing the first violation. Does **not** check sheet existence — that is
/// the caller's job (it needs the sheet set).
pub fn parse_canonical_ref(r: &str) -> Result<ParsedRef, String> {
    // Split sheet from the A1 part at the last unescaped `!`. The sheet part is
    // either a bare identifier or a single-quoted string with doubled `''`.
    let (sheet, a1_part) = split_sheet_ref(r)?;

    // The A1 part is `A1` or `A1:B2` with no `$`.
    let (start_raw, end_raw) = match a1_part.split_once(':') {
        Some((a, b)) => (a, Some(b)),
        None => (a1_part.as_str(), None),
    };
    let start =
        parse_a1(start_raw).ok_or_else(|| format!("ref {r:?} has an invalid start cell"))?;

    match end_raw {
        None => { /* single-cell form */ }
        Some(end_raw) => {
            let end =
                parse_a1(end_raw).ok_or_else(|| format!("ref {r:?} has an invalid end cell"))?;
            // Canonical: top-left first, and a degenerate range collapses to a
            // single cell — so a two-cell form with equal endpoints, or with
            // start not strictly above-left, is *not* canonical.
            if start.row == end.row && start.column == end.column {
                return Err(format!(
                    "ref {r:?} is not canonical: a degenerate range must collapse to the \
                     single-cell form (schema spec §7)"
                ));
            }
            if start.row > end.row || start.column > end.column {
                return Err(format!(
                    "ref {r:?} is not canonical: range endpoints must be top-left first \
                     (schema spec §7)"
                ));
            }
        }
    }

    // Re-emit the canonical sheet token and compare: rejects non-canonical
    // quoting (over-quoting a bare name, under-quoting one that needs it).
    let canonical_sheet_token = quote_sheet_if_needed(&sheet);
    let canonical_a1 = match end_raw {
        None => start_raw.to_owned(),
        Some(end_raw) => format!("{start_raw}:{end_raw}"),
    };
    let canonical = format!("{canonical_sheet_token}!{canonical_a1}");
    if canonical != r {
        return Err(format!(
            "ref {r:?} is not in canonical form (expected {canonical:?}, schema spec §7)"
        ));
    }

    Ok(ParsedRef { sheet })
}

/// Splits `Sheet!A1` / `'Quoted ''Name'!A1:B2` into (unquoted sheet, a1 part).
///
/// Unlike [`parse_canonical_ref`] this only splits: it does not check the A1
/// part or canonical form. That is what a sheet rename needs — it repoints the
/// sheet token of a `ref` it is not otherwise judging, and pairing this with
/// [`quote_sheet_if_needed`] keeps an already-canonical `ref` canonical.
pub fn split_sheet_ref(r: &str) -> Result<(String, String), String> {
    let bytes = r.as_bytes();
    if bytes.first() == Some(&b'\'') {
        // Quoted sheet: scan for the closing quote, treating `''` as an escaped
        // single quote inside the name.
        let mut sheet = String::new();
        let mut i = 1;
        loop {
            match bytes.get(i) {
                None => return Err(format!("ref {r:?} has an unterminated quoted sheet name")),
                Some(&b'\'') => {
                    if bytes.get(i + 1) == Some(&b'\'') {
                        sheet.push('\'');
                        i += 2;
                    } else {
                        i += 1; // closing quote
                        break;
                    }
                }
                Some(_) => {
                    // Advance by a whole UTF-8 char.
                    let ch = r[i..].chars().next().unwrap();
                    sheet.push(ch);
                    i += ch.len_utf8();
                }
            }
        }
        if bytes.get(i) != Some(&b'!') {
            return Err(format!("ref {r:?} must separate sheet and cell with '!'"));
        }
        Ok((sheet, r[i + 1..].to_owned()))
    } else {
        match r.split_once('!') {
            Some((sheet, a1)) => Ok((sheet.to_owned(), a1.to_owned())),
            None => Err(format!("ref {r:?} must be sheet-qualified (Sheet!A1)")),
        }
    }
}

/// Emits the canonical sheet token: bare if the name is a bare identifier and
/// does not parse as an A1 address; otherwise single-quoted with embedded `'`
/// doubled (schema spec §7).
pub fn quote_sheet_if_needed(sheet: &str) -> String {
    if is_bare_identifier(sheet) && !looks_like_a1(sheet) {
        sheet.to_owned()
    } else {
        format!("'{}'", sheet.replace('\'', "''"))
    }
}
