use nom::{
    IResult, Parser,
    bytes::complete::{take_while, take_while1},
    character::complete::char,
    combinator::{map, opt, recognize},
    number::complete::double,
    sequence::{delimited, pair},
};

use crate::types::ErrorKind;

/// Byte offset of `sub` within `full`. Both must be slices of the same allocation.
pub fn offset(full: &str, sub: &str) -> usize {
    sub.as_ptr() as usize - full.as_ptr() as usize
}

/// Parse a `$`-optional cell-address token: `A1`, `$A1`, `A$1`, `$A$1`.
/// Requires at least one literal `$` to match, so it never competes with the
/// plain `identifier()` grammar below — every string this *can* match was
/// previously always a parse error, since `$` appears nowhere else in the
/// grammar. On failure, the input is returned untouched.
pub fn dollar_cell_ref(i: &str) -> IResult<&str, &str> {
    let (rest, matched) = recognize((
        opt(char('$')),
        take_while1(|c: char| c.is_ascii_alphabetic()),
        opt(char('$')),
        take_while1(|c: char| c.is_ascii_digit()),
    ))
    .parse(i)?;
    if !matched.contains('$') {
        return Err(nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Tag)));
    }
    Ok((rest, matched))
}

/// Parse a float/int literal.
pub fn number_literal(i: &str) -> IResult<&str, f64> {
    double(i)
}

/// Parse a double-quoted string. Returns the inner content (no quotes).
/// NOTE: Excel-style escaped quotes (`""`) are not yet supported.
/// A string ends at the first unescaped `"`.
pub fn string_literal(i: &str) -> IResult<&str, String> {
    let mut parser = map(
        delimited(char('"'), take_while(|c| c != '"'), char('"')),
        |s: &str| s.to_string(),
    );
    parser.parse(i)
}

/// Parse TRUE or FALSE (case-insensitive), ensuring no trailing alphanumeric char
/// (so "TRUNC" is not parsed as "TRUE" + "NC") and not followed by `(` (so
/// "FALSE()" is parsed as a function call, not a bare boolean literal).
pub fn bool_literal(i: &str) -> IResult<&str, bool> {
    let upper: String = i.chars().take(5).collect::<String>().to_uppercase();
    if upper.starts_with("FALSE") {
        let rest = &i[5..];
        let next = rest.chars().next();
        if next.map(|c| c.is_alphanumeric() || c == '_' || c == '(').unwrap_or(false) {
            return Err(nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Tag)));
        }
        return Ok((rest, false));
    }
    let upper4: String = i.chars().take(4).collect::<String>().to_uppercase();
    if upper4 == "TRUE" {
        let rest = &i[4..];
        let next = rest.chars().next();
        if next.map(|c| c.is_alphanumeric() || c == '_' || c == '(').unwrap_or(false) {
            return Err(nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Tag)));
        }
        return Ok((rest, true));
    }
    Err(nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Tag)))
}

/// Parse one of the error-literal tokens (`#REF!`, `#DIV/0!`, `#NAME?`,
/// `#VALUE!`, `#NUM!`, `#N/A`, `#NULL!` — `ErrorKind::LITERAL_KINDS`) into its
/// `ErrorKind`, so a formula can embed an error value directly (`=#REF!`,
/// `=#REF!+1`) the same way it embeds a number or boolean literal.
///
/// Word-boundary rule: the character immediately following the matched
/// literal, if any, must not be alphanumeric or `_`. This rejects a
/// hypothetical trailing character glued onto the literal (`#REF!X`) rather
/// than silently truncating it to `#REF!` and leaving `X` as garbage for the
/// caller to trip over later. `#N/A` is the one literal with no trailing
/// punctuation to anchor on — the same boundary check still applies, using
/// the character after the final `A`.
///
/// None of the seven canonical strings is a prefix of another (they all
/// diverge by their second or third character), so at most one candidate can
/// ever match a given input; the loop below does not need longest-match
/// ordering.
pub fn error_literal(i: &str) -> IResult<&str, ErrorKind> {
    for kind in ErrorKind::LITERAL_KINDS {
        let text = kind.to_string();
        if let Some(rest) = i.strip_prefix(text.as_str()) {
            let boundary_ok = !rest.chars().next().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
            if boundary_ok {
                return Ok((rest, kind));
            }
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Tag)))
}

/// Parse an identifier: `[a-zA-Z_][a-zA-Z0-9_.]*`
/// Dots are allowed within identifiers to support function names like `ERROR.TYPE`.
pub fn identifier(i: &str) -> IResult<&str, &str> {
    let mut parser = recognize(pair(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '.'),
    ));
    parser.parse(i)
}

#[cfg(test)]
mod tests;
