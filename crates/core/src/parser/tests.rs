use super::*;
use super::parse_formula as parse;
use crate::parser::ast::{BinaryOp, Expr, UnaryOp};
use crate::types::ErrorKind;

#[test]
fn parse_number_literal() {
    let expr = parse("=42").unwrap();
    assert!(matches!(expr, Expr::Number(n, _) if n == 42.0));
}

#[test]
fn parse_binary_add() {
    let expr = parse("=1+2").unwrap();
    assert!(matches!(expr, Expr::BinaryOp { op: BinaryOp::Add, .. }));
}

#[test]
fn parse_precedence() {
    // 2+3*4 should parse as 2+(3*4)
    let expr = parse("=2+3*4").unwrap();
    match expr {
        Expr::BinaryOp { op: BinaryOp::Add, right, .. } => {
            assert!(matches!(*right, Expr::BinaryOp { op: BinaryOp::Mul, .. }));
        }
        _ => panic!("Expected Add at top"),
    }
}

#[test]
fn parse_function_call() {
    let expr = parse("=SUM(1,2,3)").unwrap();
    match expr {
        Expr::FunctionCall { name, args, .. } => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 3);
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn parse_function_call_arg_spans_no_leading_space() {
    // A BinaryOp argument in 2nd+ position must span exactly its own
    // tokens — not the whitespace after the preceding comma (issue #746).
    let src = "=SUM(X1*2, X2*3, X3*4)";
    let expr = parse(src).unwrap();
    let slice = |s: &Span| &src[s.offset..s.offset + s.length];
    match expr {
        Expr::FunctionCall { name, args, .. } => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 3);
            assert_eq!(slice(args[0].span()), "X1*2");
            assert_eq!(slice(args[1].span()), "X2*3");
            assert_eq!(slice(args[2].span()), "X3*4");
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn parse_function_call_arg_spans_no_space_after_comma_unaffected() {
    // Gap noted in review: when there's no whitespace after the comma at
    // all (`after_ws == after_comma`), the fix must be a no-op — args
    // still slice to exactly their own tokens.
    let src = "=SUM(X1*2,X2*3)";
    let expr = parse(src).unwrap();
    let slice = |s: &Span| &src[s.offset..s.offset + s.length];
    match expr {
        Expr::FunctionCall { name, args, .. } => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 2);
            assert_eq!(slice(args[0].span()), "X1*2");
            assert_eq!(slice(args[1].span()), "X2*3");
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn parse_parenthesised_expr_span_no_leading_space() {
    // A padded grouping `( A1 + B1 )` must not leak the space after '(' into
    // the inner expression's span (issue #751 — same class as #746/#748/#749).
    let src = "=( A1 + B1 )";
    let expr = parse(src).unwrap();
    assert!(matches!(expr, Expr::BinaryOp { .. }));
    let sp = expr.span();
    assert_eq!(&src[sp.offset..sp.offset + sp.length], "A1 + B1");
    // Extra whitespace + a different precedence level, still clean.
    let src2 = "=(  A1*B1  )";
    let expr2 = parse(src2).unwrap();
    let sp2 = expr2.span();
    assert_eq!(&src2[sp2.offset..sp2.offset + sp2.length], "A1*B1");
}

#[test]
fn parse_function_call_nested_and_unary_arg_spans_unaffected() {
    // Nested-call and unary args were already correct — confirm the
    // leading-whitespace fix doesn't disturb them.
    let src = "=SUM(1, MAX(2,3), 4)";
    let expr = parse(src).unwrap();
    let slice = |s: &Span| &src[s.offset..s.offset + s.length];
    match expr {
        Expr::FunctionCall { name, args, .. } => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 3);
            assert_eq!(slice(args[0].span()), "1");
            assert_eq!(slice(args[1].span()), "MAX(2,3)");
            assert_eq!(slice(args[2].span()), "4");
        }
        _ => panic!("Expected FunctionCall"),
    }

    let src2 = "=SUM(1, -2)";
    let expr2 = parse(src2).unwrap();
    let slice2 = |s: &Span| &src2[s.offset..s.offset + s.length];
    match expr2 {
        Expr::FunctionCall { name, args, .. } => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 2);
            assert_eq!(slice2(args[0].span()), "1");
            assert_eq!(slice2(args[1].span()), "-2");
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn parse_percent() {
    let expr = parse("=50%").unwrap();
    assert!(matches!(expr, Expr::UnaryOp { op: UnaryOp::Percent, .. }));
}

#[test]
fn parse_string_literal() {
    let expr = parse("=\"hello\"").unwrap();
    assert!(matches!(expr, Expr::Text(ref s, _) if s == "hello"));
}

#[test]
fn parse_concat_op() {
    let expr = parse("=\"a\"&\"b\"").unwrap();
    assert!(matches!(expr, Expr::BinaryOp { op: BinaryOp::Concat, .. }));
}

#[test]
#[allow(deprecated)]
fn validate_incomplete_fails() {
    let err = validate("=SUM(1,").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn parse_nested() {
    assert!(parse("=ROUND(SUM(1,2)*1.1, 1)").is_ok());
}

#[test]
fn parse_boolean() {
    let expr = parse("=TRUE").unwrap();
    assert!(matches!(expr, Expr::Bool(true, _)));
}

#[test]
fn parse_variable() {
    let expr = parse("=myVar").unwrap();
    assert!(matches!(expr, Expr::Variable(ref n, _) if n == "myVar"));
}

#[test]
fn parse_array_literal_numbers() {
    let expr = parse("={1,2,3}").unwrap();
    match expr {
        Expr::Array(elems, _) => assert_eq!(elems.len(), 3),
        _ => panic!("Expected Array"),
    }
}

#[test]
fn parse_array_literal_mixed() {
    let expr = parse("={1,\"hello\",TRUE}").unwrap();
    assert!(matches!(expr, Expr::Array(_, _)));
}

#[test]
fn parse_array_literal_empty() {
    let expr = parse("={}").unwrap();
    assert!(matches!(expr, Expr::Array(ref e, _) if e.is_empty()));
}

#[test]
fn parse_array_literal_row_spans() {
    // Each row's span must cover only that row's own elements, not the
    // whole `{...}` body (issue #745).
    let src = "={1,2;3,4}";
    let expr = parse(src).unwrap();
    let slice = |s: &Span| &src[s.offset..s.offset + s.length];

    let (rows, outer_span) = match &expr {
        Expr::Array(rows, span) => (rows, span),
        _ => panic!("Expected outer Array"),
    };
    assert_eq!(slice(outer_span), "{1,2;3,4}");
    assert_eq!(rows.len(), 2);

    let (row0_elems, row0_span) = match &rows[0] {
        Expr::Array(elems, span) => (elems, span),
        _ => panic!("Expected row 0 to be an Array"),
    };
    assert_eq!(slice(row0_span), "1,2");
    assert_eq!(row0_elems.len(), 2);
    assert_eq!(slice(row0_elems[0].span()), "1");
    assert_eq!(slice(row0_elems[1].span()), "2");

    let (row1_elems, row1_span) = match &rows[1] {
        Expr::Array(elems, span) => (elems, span),
        _ => panic!("Expected row 1 to be an Array"),
    };
    assert_eq!(slice(row1_span), "3,4");
    assert_eq!(row1_elems.len(), 2);
    assert_eq!(slice(row1_elems[0].span()), "3");
    assert_eq!(slice(row1_elems[1].span()), "4");
}

#[test]
fn parse_array_literal_single_row_unaffected() {
    // A single-row array (no semicolons) returns a flat Vec<Expr> — no
    // row-wrapper Array nodes — and must be unaffected by the row-span
    // fix (issue #745).
    let src = "={1,2,3}";
    let expr = parse(src).unwrap();
    let slice = |s: &Span| &src[s.offset..s.offset + s.length];
    match &expr {
        Expr::Array(elems, span) => {
            assert_eq!(slice(span), "{1,2,3}");
            assert_eq!(elems.len(), 3);
            assert_eq!(slice(elems[0].span()), "1");
            assert_eq!(slice(elems[1].span()), "2");
            assert_eq!(slice(elems[2].span()), "3");
            for e in elems {
                assert!(!matches!(e, Expr::Array(_, _)), "single-row elements must not be row-wrapped");
            }
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn parse_array_literal_element_no_leading_space() {
    // A compound (BinaryOp) element after a `;` (or `,`) separator must
    // span exactly its own tokens — not the whitespace after the
    // separator. Same root cause as issue #746's function-argument bug,
    // here in the sibling `parse_array_elements`.
    let src = "={1; 2+3}";
    let expr = parse(src).unwrap();
    let slice = |s: &Span| &src[s.offset..s.offset + s.length];

    let rows = match &expr {
        Expr::Array(rows, _) => rows,
        _ => panic!("Expected outer Array"),
    };
    assert_eq!(rows.len(), 2);
    let row1_elems = match &rows[1] {
        Expr::Array(elems, _) => elems,
        _ => panic!("Expected row 1 to be an Array"),
    };
    assert_eq!(row1_elems.len(), 1);
    assert_eq!(slice(row1_elems[0].span()), "2+3");
}

#[test]
fn parse_array_literal_multi_row_element_no_leading_space() {
    // Same as above, but with a compound element after a `,` in each
    // row of a multi-row array — every compound element must slice
    // minimally, with no leading whitespace from its separator.
    let src = "={1, 2+3; 4, 5*6}";
    let expr = parse(src).unwrap();
    let slice = |s: &Span| &src[s.offset..s.offset + s.length];

    let rows = match &expr {
        Expr::Array(rows, _) => rows,
        _ => panic!("Expected outer Array"),
    };
    assert_eq!(rows.len(), 2);

    let row0_elems = match &rows[0] {
        Expr::Array(elems, _) => elems,
        _ => panic!("Expected row 0 to be an Array"),
    };
    assert_eq!(slice(row0_elems[0].span()), "1");
    assert_eq!(slice(row0_elems[1].span()), "2+3");

    let row1_elems = match &rows[1] {
        Expr::Array(elems, _) => elems,
        _ => panic!("Expected row 1 to be an Array"),
    };
    assert_eq!(slice(row1_elems[0].span()), "4");
    assert_eq!(slice(row1_elems[1].span()), "5*6");
}

#[test]
fn parse_array_literal_single_element_rows() {
    // Gap noted in review: single-element-per-row arrays must produce
    // flat one-element rows with correct spans.
    let src = "={1;2}";
    let expr = parse(src).unwrap();
    let slice = |s: &Span| &src[s.offset..s.offset + s.length];

    let rows = match &expr {
        Expr::Array(rows, span) => {
            assert_eq!(slice(span), "{1;2}");
            rows
        }
        _ => panic!("Expected outer Array"),
    };
    assert_eq!(rows.len(), 2);

    let (row0_elems, row0_span) = match &rows[0] {
        Expr::Array(elems, span) => (elems, span),
        _ => panic!("Expected row 0 to be an Array"),
    };
    assert_eq!(slice(row0_span), "1");
    assert_eq!(row0_elems.len(), 1);
    assert_eq!(slice(row0_elems[0].span()), "1");

    let (row1_elems, row1_span) = match &rows[1] {
        Expr::Array(elems, span) => (elems, span),
        _ => panic!("Expected row 1 to be an Array"),
    };
    assert_eq!(slice(row1_span), "2");
    assert_eq!(row1_elems.len(), 1);
    assert_eq!(slice(row1_elems[0].span()), "2");
}

#[test]
fn parse_array_in_function_call() {
    let expr = parse("=SUM({1,2,3})").unwrap();
    match expr {
        Expr::FunctionCall { name, args, .. } => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], Expr::Array(_, _)));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn parse_power_right_assoc() {
    // 2^3^2 = 2^(3^2) = 2^9 = 512 (right-associative)
    let expr = parse("=2^3^2").unwrap();
    match expr {
        Expr::BinaryOp { op: BinaryOp::Pow, right, .. } => {
            assert!(matches!(*right, Expr::BinaryOp { op: BinaryOp::Pow, .. }));
        }
        _ => panic!("Expected Pow at top"),
    }
}

// ── error literal (issue #716) ──────────────────────────────────────────

#[test]
fn parse_error_literal_ref() {
    let expr = parse("=#REF!").unwrap();
    assert!(matches!(expr, Expr::Error(ErrorKind::Ref, _)));
}

#[test]
fn parse_error_literal_every_canonical_form() {
    let cases = [
        ("=#REF!", ErrorKind::Ref),
        ("=#DIV/0!", ErrorKind::DivByZero),
        ("=#NAME?", ErrorKind::Name),
        ("=#VALUE!", ErrorKind::Value),
        ("=#NUM!", ErrorKind::Num),
        ("=#N/A", ErrorKind::NA),
        ("=#NULL!", ErrorKind::Null),
    ];
    for (formula, expected) in cases {
        let expr = parse(formula).unwrap_or_else(|e| panic!("{formula} failed to parse: {e}"));
        assert!(matches!(&expr, Expr::Error(k, _) if *k == expected), "{formula} -> {expr:?}");
    }
}

#[test]
fn parse_error_literal_span_covers_exactly_the_literal_text() {
    let src = "=#REF!";
    let expr = parse(src).unwrap();
    let sp = expr.span();
    assert_eq!(&src[sp.offset..sp.offset + sp.length], "#REF!");
}

#[test]
fn parse_error_literal_propagates_through_binary_op() {
    // `=#REF!+1` (issue #716 acceptance): the literal is a normal primary, so
    // it composes with the rest of the grammar like any other literal.
    let expr = parse("=#REF!+1").unwrap();
    match expr {
        Expr::BinaryOp { op: BinaryOp::Add, left, .. } => {
            assert!(matches!(*left, Expr::Error(ErrorKind::Ref, _)));
        }
        other => panic!("Expected Add at top, got {other:?}"),
    }
}

#[test]
fn parse_error_literal_na_has_no_trailing_punctuation() {
    // #N/A is the one canonical form with no trailing '!'/'?' -- must still
    // parse standalone and compose with a following operator.
    assert!(matches!(parse("=#N/A").unwrap(), Expr::Error(ErrorKind::NA, _)));
    match parse("=#N/A+1").unwrap() {
        Expr::BinaryOp { op: BinaryOp::Add, left, .. } => {
            assert!(matches!(*left, Expr::Error(ErrorKind::NA, _)));
        }
        other => panic!("Expected Add at top, got {other:?}"),
    }
}

#[test]
fn parse_error_literal_rejects_over_matched_trailing_text() {
    // A hypothetical trailing character glued onto the literal (`#REF!X`)
    // must not silently parse as `#REF!` followed by dropped garbage -- the
    // whole formula should fail to parse.
    assert!(parse("=#REF!X").is_err());
    assert!(parse("=#N/AX").is_err());
}

#[test]
fn parse_error_literal_unsupported_is_not_a_literal() {
    // `#UNSUPPORTED!` is engine-internal, never accepted as formula text.
    assert!(parse("=#UNSUPPORTED!").is_err());
}

#[test]
fn parse_error_literal_is_case_insensitive() {
    // Matches the parser's existing case-insensitive keywords (TRUE/FALSE,
    // function names, cell references).
    assert!(matches!(parse("=#ref!").unwrap(), Expr::Error(ErrorKind::Ref, _)));
    assert!(matches!(parse("=#Div/0!").unwrap(), Expr::Error(ErrorKind::DivByZero, _)));
    assert!(matches!(parse("=#n/a").unwrap(), Expr::Error(ErrorKind::NA, _)));
}

#[test]
fn parses_whole_column_table_ref() {
    let expr = parse("=SUM(Recipe[reference_per_100g])").unwrap();
    match expr {
        Expr::FunctionCall { name, args, .. } => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 1);
            match &args[0] {
                Expr::Reference(Ref::Table { table, column, this_row }, _) => {
                    assert_eq!(table.as_deref(), Some("Recipe"));
                    assert_eq!(column, "reference_per_100g");
                    assert!(!this_row);
                }
                other => panic!("expected Ref::Table, got {other:?}"),
            }
        }
        other => panic!("expected FunctionCall, got {other:?}"),
    }
}

#[test]
fn parses_current_row_qualified_table_ref() {
    let expr = parse("=Recipe[@quantity_g]").unwrap();
    match expr {
        Expr::Reference(Ref::Table { table, column, this_row }, _) => {
            assert_eq!(table.as_deref(), Some("Recipe"));
            assert_eq!(column, "quantity_g");
            assert!(this_row);
        }
        other => panic!("expected Ref::Table, got {other:?}"),
    }
}

#[test]
fn table_ref_in_binary_expr() {
    let expr = parse("=Recipe[@a]*Recipe[@b]/100").unwrap();
    // Just confirm it parses to a BinaryOp tree without error; the two
    // Ref::Table cases are already covered above.
    assert!(matches!(expr, Expr::BinaryOp { .. }));
}
