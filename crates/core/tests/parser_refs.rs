//! P1.2 reference grammar — parse-level tests (issue #524).
//!
//! Covers sheet-qualified cell refs (`Sheet1!A1`), quoted sheet names
//! (`'Quoted Name'!A1:B2`), cross-sheet ranges, the `Ref` model, canonical
//! display, and the zero-behavior-change guarantee for bare `A1` / `A1:D4` /
//! named identifiers (which stay `Expr::Variable`).
//!
//! Evaluation semantics for these references (e.g. `#REF!` for a missing
//! sheet) are pipeline-verified via the P1.5 workbook fixtures and land with
//! the P1.3 resolver; nothing here self-confirms evaluation values.

use truecalc_core::{CellAddr, Engine, Expr, Ref};

fn parse(formula: &str) -> Expr {
    Engine::sheets()
        .parse(formula)
        .unwrap_or_else(|e| panic!("expected {formula:?} to parse: {e}"))
}

fn parse_err(formula: &str) {
    assert!(
        Engine::sheets().parse(formula).is_err(),
        "expected {formula:?} to fail to parse"
    );
}

fn addr(col: u32, row: u32) -> CellAddr {
    CellAddr::new(col, row)
}

// ── sheet-qualified cells ───────────────────────────────────────────────────

#[test]
fn sheet_qualified_cell() {
    match parse("=Sheet1!A1") {
        Expr::Reference(Ref::Cell { sheet, addr: a }, _) => {
            assert_eq!(sheet.as_deref(), Some("Sheet1"));
            assert_eq!(a, addr(1, 1));
        }
        other => panic!("expected Reference(Cell), got {other:?}"),
    }
}

#[test]
fn sheet_qualified_range() {
    match parse("=Sheet1!A1:B2") {
        Expr::Reference(Ref::Range { sheet, start, end }, _) => {
            assert_eq!(sheet.as_deref(), Some("Sheet1"));
            assert_eq!(start, addr(1, 1));
            assert_eq!(end, addr(2, 2));
        }
        other => panic!("expected Reference(Range), got {other:?}"),
    }
}

#[test]
fn lowercase_sheet_and_cell() {
    match parse("=data!a1") {
        Expr::Reference(Ref::Cell { sheet, addr: a }, _) => {
            // Sheet case is preserved as written; resolution is delegated.
            assert_eq!(sheet.as_deref(), Some("data"));
            assert_eq!(a, addr(1, 1));
        }
        other => panic!("expected Reference(Cell), got {other:?}"),
    }
}

#[test]
fn multi_letter_columns() {
    match parse("=Sheet1!AA10:BC42") {
        Expr::Reference(Ref::Range { start, end, .. }, _) => {
            assert_eq!(start, addr(27, 10));
            assert_eq!(end, addr(55, 42));
        }
        other => panic!("expected Reference(Range), got {other:?}"),
    }
}

#[test]
fn sheet_with_underscore_and_digits() {
    match parse("=My_Sheet1!C3") {
        Expr::Reference(Ref::Cell { sheet, addr: a }, _) => {
            assert_eq!(sheet.as_deref(), Some("My_Sheet1"));
            assert_eq!(a, addr(3, 3));
        }
        other => panic!("expected Reference(Cell), got {other:?}"),
    }
}

#[test]
fn sheet_name_that_looks_like_a_cell() {
    // A sheet named "A1" is unambiguous before `!`.
    match parse("=A1!B2") {
        Expr::Reference(Ref::Cell { sheet, addr: a }, _) => {
            assert_eq!(sheet.as_deref(), Some("A1"));
            assert_eq!(a, addr(2, 2));
        }
        other => panic!("expected Reference(Cell), got {other:?}"),
    }
}

#[test]
fn missing_sheet_still_parses() {
    // Whether the sheet exists is an evaluation concern (#REF! via the
    // resolver, P1.3), not a parse error.
    assert!(matches!(
        parse("=MissingSheet!A1"),
        Expr::Reference(Ref::Cell { .. }, _)
    ));
}

// ── quoted sheet names ──────────────────────────────────────────────────────

#[test]
fn quoted_sheet_cell() {
    match parse("='Quoted Name'!A1") {
        Expr::Reference(Ref::Cell { sheet, addr: a }, _) => {
            assert_eq!(sheet.as_deref(), Some("Quoted Name"));
            assert_eq!(a, addr(1, 1));
        }
        other => panic!("expected Reference(Cell), got {other:?}"),
    }
}

#[test]
fn quoted_sheet_range() {
    match parse("='Quoted Name'!A1:B2") {
        Expr::Reference(Ref::Range { sheet, start, end }, _) => {
            assert_eq!(sheet.as_deref(), Some("Quoted Name"));
            assert_eq!(start, addr(1, 1));
            assert_eq!(end, addr(2, 2));
        }
        other => panic!("expected Reference(Range), got {other:?}"),
    }
}

#[test]
fn quoted_single_word_sheet() {
    // Quoting is optional for bare-identifier names but still valid.
    match parse("='Data'!A1") {
        Expr::Reference(Ref::Cell { sheet, .. }, _) => {
            assert_eq!(sheet.as_deref(), Some("Data"));
        }
        other => panic!("expected Reference(Cell), got {other:?}"),
    }
}

#[test]
fn escaped_quote_in_sheet_name() {
    match parse("='It''s'!A1") {
        Expr::Reference(Ref::Cell { sheet, .. }, _) => {
            assert_eq!(sheet.as_deref(), Some("It's"));
        }
        other => panic!("expected Reference(Cell), got {other:?}"),
    }
}

#[test]
fn unicode_quoted_sheet() {
    match parse("='Q2 Données'!A1") {
        Expr::Reference(Ref::Cell { sheet, .. }, _) => {
            assert_eq!(sheet.as_deref(), Some("Q2 Données"));
        }
        other => panic!("expected Reference(Cell), got {other:?}"),
    }
}

// ── references inside expressions ───────────────────────────────────────────

#[test]
fn refs_in_arithmetic() {
    match parse("=Data!A1+Data!B1") {
        Expr::BinaryOp { left, right, .. } => {
            assert!(matches!(*left, Expr::Reference(Ref::Cell { .. }, _)));
            assert!(matches!(*right, Expr::Reference(Ref::Cell { .. }, _)));
        }
        other => panic!("expected BinaryOp, got {other:?}"),
    }
}

#[test]
fn range_ref_as_function_arg() {
    match parse("=SUM(Data!A1:A3)") {
        Expr::FunctionCall { name, args, .. } => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], Expr::Reference(Ref::Range { .. }, _)));
        }
        other => panic!("expected FunctionCall, got {other:?}"),
    }
}

#[test]
fn ref_in_if_condition() {
    assert!(Engine::sheets().parse("=IF(Data!A1>5,\"big\",\"small\")").is_ok());
}

#[test]
fn quoted_refs_in_arithmetic() {
    assert!(Engine::sheets()
        .parse("='Quoted Name'!A1+'Quoted Name'!B1")
        .is_ok());
}

#[test]
fn excel_engine_accepts_same_grammar() {
    assert!(Engine::excel().parse("='Quoted Name'!A1:B2").is_ok());
}

// ── spans ───────────────────────────────────────────────────────────────────

#[test]
fn span_covers_whole_sheet_qualified_range() {
    let expr = parse("=Sheet1!A1:B2");
    let span = expr.span();
    assert_eq!(span.offset, 1);
    assert_eq!(span.length, "Sheet1!A1:B2".len());
}

#[test]
fn span_covers_whole_quoted_reference() {
    let expr = parse("='My Sheet'!A1");
    let span = expr.span();
    assert_eq!(span.offset, 1);
    assert_eq!(span.length, "'My Sheet'!A1".len());
}

// ── zero behavior change for bare identifiers ──────────────────────────────

#[test]
fn bare_cell_stays_variable() {
    assert!(matches!(parse("=A1"), Expr::Variable(ref n, _) if n == "A1"));
}

#[test]
fn bare_range_stays_variable() {
    assert!(matches!(parse("=A1:D4"), Expr::Variable(ref n, _) if n == "A1:D4"));
}

#[test]
fn bare_name_stays_variable() {
    assert!(matches!(parse("=TAX_RATE"), Expr::Variable(ref n, _) if n == "TAX_RATE"));
}

#[test]
fn dotted_function_names_still_parse() {
    assert!(matches!(
        parse("=ERROR.TYPE(1)"),
        Expr::FunctionCall { ref name, .. } if name == "ERROR.TYPE"
    ));
}

// ── parse errors ────────────────────────────────────────────────────────────

#[test]
fn error_missing_cell_after_bang() {
    parse_err("=Sheet1!");
}

#[test]
fn error_number_after_bang() {
    parse_err("=Sheet1!123");
}

#[test]
fn error_name_after_bang() {
    // Sheet-scoped named references are not part of the v1 grammar.
    parse_err("=Sheet1!foo");
}

#[test]
fn error_row_zero_after_bang() {
    parse_err("=Sheet1!A0");
}

#[test]
fn error_empty_quoted_sheet() {
    parse_err("=''!A1");
}

#[test]
fn error_unterminated_quoted_sheet() {
    parse_err("='Oops");
}

#[test]
fn error_quoted_sheet_without_bang() {
    parse_err("='Data'");
}

#[test]
fn error_bad_range_second_endpoint() {
    parse_err("=Data!A1:xyz");
}

#[test]
fn error_whitespace_around_bang() {
    parse_err("=Sheet1 !A1");
    parse_err("=Sheet1! A1");
}

// ── CellAddr ───────────────────────────────────────────────────────────────

#[test]
fn celladdr_parse_and_display() {
    for (text, col, row, canonical) in [
        ("A1", 1, 1, "A1"),
        ("z26", 26, 26, "Z26"),
        ("AA1", 27, 1, "AA1"),
        ("BC42", 55, 42, "BC42"),
        ("zz1", 702, 1, "ZZ1"),
    ] {
        let a = CellAddr::parse(text).unwrap_or_else(|| panic!("{text} should parse"));
        assert_eq!(a, addr(col, row), "{text}");
        assert_eq!(a.to_string(), canonical, "{text}");
    }
}

#[test]
fn celladdr_rejects_non_addresses() {
    for text in ["", "A", "1", "A0", "1A", "A1B", "A 1", "A1.5"] {
        assert!(CellAddr::parse(text).is_none(), "{text:?} should not parse");
    }
}

// ── Ref::classify (bare identifiers → named refs) ───────────────────────────

#[test]
fn classify_bare_cell() {
    assert_eq!(
        Ref::classify("A1"),
        Ref::Cell { sheet: None, addr: addr(1, 1) }
    );
}

#[test]
fn classify_bare_range() {
    assert_eq!(
        Ref::classify("A1:D4"),
        Ref::Range { sheet: None, start: addr(1, 1), end: addr(4, 4) }
    );
}

#[test]
fn classify_bare_identifier_is_name() {
    assert_eq!(Ref::classify("TAX_RATE"), Ref::Name("TAX_RATE".into()));
    assert_eq!(Ref::classify("A1:xyz"), Ref::Name("A1:xyz".into()));
}

// ── canonical display ───────────────────────────────────────────────────────

#[test]
fn display_unquoted_when_bare_identifier() {
    let r = Ref::Cell { sheet: Some("Sheet1".into()), addr: addr(1, 1) };
    assert_eq!(r.to_string(), "Sheet1!A1");
}

#[test]
fn display_quotes_when_name_needs_it() {
    let r = Ref::Cell { sheet: Some("Quoted Name".into()), addr: addr(1, 1) };
    assert_eq!(r.to_string(), "'Quoted Name'!A1");
}

#[test]
fn display_doubles_embedded_quotes() {
    let r = Ref::Cell { sheet: Some("It's".into()), addr: addr(1, 1) };
    assert_eq!(r.to_string(), "'It''s'!A1");
}

#[test]
fn display_quotes_a1_like_sheet_name() {
    let r = Ref::Cell { sheet: Some("A1".into()), addr: addr(2, 2) };
    assert_eq!(r.to_string(), "'A1'!B2");
}

#[test]
fn display_quotes_boolean_sheet_names() {
    // TRUE/FALSE parse as boolean literals before identifiers, so a sheet
    // with one of those names must be quoted in canonical form to re-parse.
    let r = Ref::Cell { sheet: Some("TRUE".into()), addr: addr(1, 1) };
    assert_eq!(r.to_string(), "'TRUE'!A1");
    let r = Ref::Cell { sheet: Some("false".into()), addr: addr(1, 1) };
    assert_eq!(r.to_string(), "'false'!A1");
}

#[test]
fn display_quotes_leading_digit_sheet_name() {
    let r = Ref::Range {
        sheet: Some("2024".into()),
        start: addr(1, 1),
        end: addr(2, 2),
    };
    assert_eq!(r.to_string(), "'2024'!A1:B2");
}

#[test]
fn display_range_without_sheet() {
    let r = Ref::Range { sheet: None, start: addr(1, 1), end: addr(4, 4) };
    assert_eq!(r.to_string(), "A1:D4");
}

#[test]
fn display_name() {
    assert_eq!(Ref::Name("TAX_RATE".into()).to_string(), "TAX_RATE");
}

#[test]
fn display_round_trips_through_parser() {
    for formula in ["='It''s'!A1:B2", "=Sheet1!A1", "='Q2 Données'!AA10", "='TRUE'!A1", "='FALSE'!B2:C3"] {
        let r1 = match parse(formula) {
            Expr::Reference(r, _) => r,
            other => panic!("expected Reference, got {other:?}"),
        };
        let printed = format!("={r1}");
        let r2 = match parse(&printed) {
            Expr::Reference(r, _) => r,
            other => panic!("expected Reference on round-trip, got {other:?}"),
        };
        assert_eq!(r1, r2, "round-trip of {formula}");
    }
}
