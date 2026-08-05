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

// ── error_literal (issue #716) ──────────────────────────────────────────

#[test]
fn error_literal_matches_every_canonical_form() {
    assert_eq!(error_literal("#REF!"), Ok(("", ErrorKind::Ref)));
    assert_eq!(error_literal("#DIV/0!"), Ok(("", ErrorKind::DivByZero)));
    assert_eq!(error_literal("#NAME?"), Ok(("", ErrorKind::Name)));
    assert_eq!(error_literal("#VALUE!"), Ok(("", ErrorKind::Value)));
    assert_eq!(error_literal("#NUM!"), Ok(("", ErrorKind::Num)));
    assert_eq!(error_literal("#N/A"), Ok(("", ErrorKind::NA)));
    assert_eq!(error_literal("#NULL!"), Ok(("", ErrorKind::Null)));
}

#[test]
fn error_literal_leaves_trailing_input_after_the_literal() {
    // Used mid-formula, e.g. `=#REF!+1` -- only the literal itself is
    // consumed, the rest is left for the caller (here the `+1` operator tail).
    assert_eq!(error_literal("#REF!+1"), Ok(("+1", ErrorKind::Ref)));
    assert_eq!(error_literal("#N/A+1"), Ok(("+1", ErrorKind::NA)));
    assert_eq!(error_literal("#REF! rest"), Ok((" rest", ErrorKind::Ref)));
}

#[test]
fn error_literal_rejects_unsupported_and_does_not_expose_it_as_a_literal() {
    // Unsupported is engine-internal (never produced by Sheets/Excel text),
    // so `#UNSUPPORTED!` must not parse as an error literal at all.
    assert!(error_literal("#UNSUPPORTED!").is_err());
}

#[test]
fn error_literal_rejects_over_match_with_a_trailing_identifier_char() {
    // A hypothetical trailing character glued onto the literal must not be
    // silently truncated away -- the whole token should fail to match so the
    // caller sees the extra text rather than a wrongly-scoped error literal.
    for text in ["#REF!X", "#DIV/0!X", "#NAME?X", "#VALUE!X", "#NUM!X", "#NULL!X"] {
        assert!(error_literal(text).is_err(), "{text:?} should not match");
    }
}

#[test]
fn error_literal_na_has_no_trailing_punctuation_but_still_word_bounds() {
    // #N/A is the one literal that ends on a plain letter, not `!`/`?` --
    // it must still be accepted on its own (under-match would wrongly
    // reject it for lacking a trailing exclamation mark)...
    assert_eq!(error_literal("#N/A"), Ok(("", ErrorKind::NA)));
    // ...while still refusing to over-match into a longer identifier-like tail.
    assert!(error_literal("#N/AX").is_err());
    assert!(error_literal("#N/A1").is_err());
}

#[test]
fn error_literal_rejects_unrecognized_text() {
    assert!(error_literal("#FOO!").is_err());
    assert!(error_literal("REF!").is_err()); // missing leading '#'
    assert!(error_literal("").is_err());
}

#[test]
fn error_literal_is_case_insensitive_like_bool_literal() {
    // Matches this file's convention (see `booleans`/`bool_literal`): keyword
    // tokens accept any casing.
    assert_eq!(error_literal("#ref!"), Ok(("", ErrorKind::Ref)));
    assert_eq!(error_literal("#Ref!"), Ok(("", ErrorKind::Ref)));
    assert_eq!(error_literal("#div/0!"), Ok(("", ErrorKind::DivByZero)));
    assert_eq!(error_literal("#n/a"), Ok(("", ErrorKind::NA)));
    assert_eq!(error_literal("#N/a rest"), Ok((" rest", ErrorKind::NA)));
    // Boundary check still applies regardless of case.
    assert!(error_literal("#ref!x").is_err());
}
