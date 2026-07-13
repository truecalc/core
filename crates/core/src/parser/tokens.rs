use nom::{
    IResult, Parser,
    bytes::complete::{take_while, take_while1},
    character::complete::char,
    combinator::{map, opt, recognize},
    number::complete::double,
    sequence::{delimited, pair},
};

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
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(number_literal("3.14rest"), Ok(("rest", 3.14)));
        assert_eq!(number_literal("42"), Ok(("", 42.0)));
        assert_eq!(number_literal("1e3"), Ok(("", 1000.0)));
    }

    #[test]
    fn strings() {
        assert_eq!(string_literal("\"hello\""), Ok(("", "hello".to_string())));
        assert_eq!(string_literal("\"\""), Ok(("", "".to_string())));
        // Unterminated string returns an error
        assert!(string_literal("\"unterminated").is_err());
    }

    #[test]
    fn booleans() {
        assert_eq!(bool_literal("TRUE"), Ok(("", true)));
        assert_eq!(bool_literal("false"), Ok(("", false)));
        assert_eq!(bool_literal("FALSE rest"), Ok((" rest", false)));
        assert!(bool_literal("TRUNC(1)").is_err());
        assert!(bool_literal("TRUENESS").is_err());
    }

    #[test]
    fn identifiers() {
        assert_eq!(identifier("myVar"), Ok(("", "myVar")));
        assert_eq!(identifier("_x1 rest"), Ok((" rest", "_x1")));
        assert!(identifier("123abc").is_err());
    }

    #[test]
    fn dollar_cell_ref_matches_all_three_dollar_shapes() {
        assert_eq!(dollar_cell_ref("$A$1 rest"), Ok((" rest", "$A$1")));
        assert_eq!(dollar_cell_ref("$A1 rest"), Ok((" rest", "$A1")));
        assert_eq!(dollar_cell_ref("A$1 rest"), Ok((" rest", "A$1")));
    }

    #[test]
    fn dollar_cell_ref_rejects_plain_no_dollar_input() {
        // No '$' present -> must fail, leaving `identifier()` to own this case.
        assert!(dollar_cell_ref("A1").is_err());
        assert!(dollar_cell_ref("myVar").is_err());
    }

    #[test]
    fn dollar_cell_ref_no_match_leaves_input_unchanged() {
        match dollar_cell_ref("A1") {
            Err(nom::Err::Error(e)) => assert_eq!(e.input, "A1"),
            other => panic!("expected Error with unchanged input, got {other:?}"),
        }
    }

    #[test]
    fn dollar_cell_ref_rejects_malformed_shapes() {
        for text in ["$", "$$A1", "$1A"] {
            assert!(dollar_cell_ref(text).is_err(), "{text:?} should not match");
        }
    }

    #[test]
    fn offset_calc() {
        let full = "=SUM(1,2)";
        let sub = &full[5..]; // "1,2)"
        assert_eq!(offset(full, sub), 5);
    }

    #[test]
    fn bool_boundary() {
        // FALSE branch word-boundary rejection
        assert!(bool_literal("falsetto").is_err());
        // TRUE branch already tested in booleans test
    }

    #[test]
    fn offset_boundaries() {
        let full = "=SUM(1,2)";
        assert_eq!(offset(full, full), 0);
        assert_eq!(offset(full, &full[full.len()..]), full.len());
    }
}
