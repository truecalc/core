pub mod ast;
pub mod refs;
pub mod tokens;

pub use ast::{Expr, Span};
pub use refs::{CellAddr, Ref};
use ast::{BinaryOp, UnaryOp};
use crate::types::ParseError;
use nom::{IResult, character::complete::multispace0};
use tokens::{bool_literal, dollar_cell_ref, identifier, number_literal, offset, string_literal};

/// A cell-address token: `dollar_cell_ref()`'s `$`-bearing shape (`$A1`,
/// `A$1`, `$A$1`), or — when no literal `$` is present — `identifier()`'s
/// plain shape (`A1`). `dollar_cell_ref` must be tried first: `identifier`
/// does not fail on input like `"A$1"`, it just stops early at `A` and
/// succeeds, so trying it first would never give `dollar_cell_ref` a chance
/// to claim the `$1` suffix. Used wherever a range endpoint is expected, so
/// either corner of a range may independently carry `$` anchors (e.g.
/// `A1:$D$4`).
fn cell_ref_text(i: &str) -> IResult<&str, &str> {
    dollar_cell_ref(i).or_else(|_| identifier(i))
}

struct Parser<'a> {
    full: &'a str,
}

impl<'a> Parser<'a> {
    fn new(full: &'a str) -> Self {
        Self { full }
    }

    fn span(&self, before: &str, after: &str) -> Span {
        let start = offset(self.full, before);
        let end = offset(self.full, after);
        Span::new(start, end - start)
    }

    // ── primary ────────────────────────────────────────────────────────────

    fn parse_primary(&self, i: &'a str) -> IResult<&'a str, Expr> {
        let i = multispace0(i)?.0;

        // Number literal (must come before identifier to catch e.g. "1e3")
        if let Ok((rest, n)) = number_literal(i) {
            return Ok((rest, Expr::Number(n, self.span(i, rest))));
        }

        // String literal
        if let Ok((rest, text)) = string_literal(i) {
            return Ok((rest, Expr::Text(text, self.span(i, rest))));
        }

        // Array literal: {expr, expr, ...}
        if let Some(inner) = i.strip_prefix('{') {
            let (rest, elems) = self.parse_array_elements(inner)?;
            let rest = multispace0(rest)?.0;
            if let Some(after) = rest.strip_prefix('}') {
                return Ok((after, Expr::Array(elems, self.span(i, after))));
            }
            return Err(nom::Err::Error(nom::error::Error::new(
                rest,
                nom::error::ErrorKind::Char,
            )));
        }

        // Parenthesised expression
        if let Some(inner) = i.strip_prefix('(') {
            let (rest, expr) = self.parse_comparison(inner)?;
            let rest = multispace0(rest)?.0;
            if let Some(after) = rest.strip_prefix(')') {
                return Ok((after, expr));
            }
            return Err(nom::Err::Error(nom::error::Error::new(
                rest,
                nom::error::ErrorKind::Char,
            )));
        }

        // Boolean (before identifier — uses word-boundary check in bool_literal)
        if let Ok((rest, b)) = bool_literal(i) {
            return Ok((rest, Expr::Bool(b, self.span(i, rest))));
        }

        // Quoted-sheet reference: 'Sheet Name'!A1 / 'Sheet Name'!A1:B2
        if i.starts_with('\'') {
            return self.parse_quoted_sheet_ref(i);
        }

        // $-anchored cell reference (bare, no sheet): $A$1, $A1, A$1. A
        // '$'-bearing token can only ever be a cell/range reference (never a
        // sheet name, function call, or plain variable — none of those can
        // contain '$'), so it short-circuits straight to Expr::Variable,
        // mirroring how plain `A1` becomes Expr::Variable("A1", ..) below.
        if let Ok((rest, span)) = dollar_cell_ref(i) {
            let rest_ws = multispace0(rest)?.0;
            if let Some(after_colon) = rest_ws.strip_prefix(':') {
                if let Ok((rest2, end_span)) = cell_ref_text(after_colon) {
                    if CellAddr::parse(end_span).is_some() {
                        let range_name = format!("{}:{}", span, end_span);
                        return Ok((rest2, Expr::Variable(range_name, self.span(i, rest2))));
                    }
                }
            }
            return Ok((rest, Expr::Variable(span.to_string(), self.span(i, rest))));
        }

        // Identifier: sheet-qualified reference, variable, or function call
        if let Ok((rest, name)) = identifier(i) {
            // Sheet-qualified reference: Sheet1!A1 / Sheet1!A1:B2 — `!` binds
            // tightly to the sheet name (no whitespace on either side).
            if let Some(after_bang) = rest.strip_prefix('!') {
                return self.parse_ref_body(i, name.to_string(), after_bang);
            }
            let rest_ws = multispace0(rest)?.0;
            if let Some(args_input) = rest_ws.strip_prefix('(') {
                // Function call
                let (rest2, args) = self.parse_arg_list(args_input)?;
                let rest2 = multispace0(rest2)?.0;
                if let Some(after_close) = rest2.strip_prefix(')') {
                    let func_expr = Expr::FunctionCall {
                        name: name.to_uppercase(),
                        args,
                        span: self.span(i, after_close),
                    };
                    // Detect immediately-invoked call: FUNC(lambda_args)(call_args)
                    let after_ws = multispace0(after_close)?.0;
                    if let Some(call_input) = after_ws.strip_prefix('(') {
                        let (rest3, call_args) = self.parse_arg_list(call_input)?;
                        let rest3 = multispace0(rest3)?.0;
                        if let Some(after) = rest3.strip_prefix(')') {
                            return Ok((after, Expr::Apply {
                                func: Box::new(func_expr),
                                call_args,
                                span: self.span(i, after),
                            }));
                        }
                        return Err(nom::Err::Error(nom::error::Error::new(
                            rest3,
                            nom::error::ErrorKind::Char,
                        )));
                    }
                    return Ok((after_close, func_expr));
                }
                return Err(nom::Err::Error(nom::error::Error::new(
                    rest2,
                    nom::error::ErrorKind::Char,
                )));
            }
            // Range reference: A1:D4 (end corner may itself be $-anchored,
            // e.g. A1:$D$4 — validated via the same CellAddr::parse used
            // for the dollar-led branch above, so both paths agree on shape).
            if CellAddr::parse(name).is_some() {
                if let Some(after_colon) = rest_ws.strip_prefix(':') {
                    if let Ok((rest2, name2)) = cell_ref_text(after_colon) {
                        if CellAddr::parse(name2).is_some() {
                            let range_name = format!("{}:{}", name, name2);
                            return Ok((rest2, Expr::Variable(range_name, self.span(i, rest2))));
                        }
                    }
                }
            }
            return Ok((rest, Expr::Variable(name.to_string(), self.span(i, rest))));
        }

        Err(nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Alt)))
    }

    // ── sheet-qualified references ──────────────────────────────────────

    /// Parse the part after `!`: a cell address, optionally `:cell` for a
    /// range. `start` is where the whole reference began (for spans); `sheet`
    /// is the unescaped sheet name.
    fn parse_ref_body(&self, start: &'a str, sheet: String, i: &'a str) -> IResult<&'a str, Expr> {
        let err = || nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Tag));
        let (rest, cell_text) = cell_ref_text(i).map_err(|_| err())?;
        let addr = CellAddr::parse(cell_text).ok_or_else(err)?;
        // Optional range tail, mirroring the bare `A1:D4` grammar below.
        let rest_ws = multispace0(rest)?.0;
        if let Some(after_colon) = rest_ws.strip_prefix(':') {
            if let Ok((rest2, end_text)) = cell_ref_text(after_colon) {
                if let Some(end) = CellAddr::parse(end_text) {
                    let r = Ref::Range { sheet: Some(sheet), start: addr, end };
                    return Ok((rest2, Expr::Reference(r, self.span(start, rest2))));
                }
            }
        }
        let r = Ref::Cell { sheet: Some(sheet), addr };
        Ok((rest, Expr::Reference(r, self.span(start, rest))))
    }

    /// Parse `'Sheet Name'!A1` / `'Sheet Name'!A1:B2`. `i` starts at the
    /// opening quote. `''` inside the quotes is an escaped single quote.
    fn parse_quoted_sheet_ref(&self, i: &'a str) -> IResult<&'a str, Expr> {
        let inner = &i[1..];
        let mut sheet = String::new();
        let mut idx = 0;
        loop {
            match inner[idx..].find('\'') {
                // Unterminated quoted sheet name
                None => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        i,
                        nom::error::ErrorKind::Char,
                    )));
                }
                Some(q) => {
                    sheet.push_str(&inner[idx..idx + q]);
                    let after = idx + q + 1;
                    if inner[after..].starts_with('\'') {
                        sheet.push('\'');
                        idx = after + 1;
                    } else {
                        idx = after;
                        break;
                    }
                }
            }
        }
        let rest = &inner[idx..];
        if sheet.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                i,
                nom::error::ErrorKind::Char,
            )));
        }
        match rest.strip_prefix('!') {
            Some(after_bang) => self.parse_ref_body(i, sheet, after_bang),
            None => Err(nom::Err::Error(nom::error::Error::new(
                rest,
                nom::error::ErrorKind::Char,
            ))),
        }
    }

    fn parse_arg_list(&self, i: &'a str) -> IResult<&'a str, Vec<Expr>> {
        let mut args = Vec::new();
        let mut rest = multispace0(i)?.0;

        if rest.starts_with(')') {
            return Ok((rest, args));
        }

        // Parse first argument (may be empty if it starts with comma or close paren)
        let ws = multispace0(rest)?.0;
        if ws.starts_with(',') || ws.starts_with(')') {
            // Empty first argument
            args.push(Expr::Variable(String::new(), Span::new(0, 0)));
        } else {
            let (r, first) = self.parse_comparison(rest)?;
            args.push(first);
            rest = r;
        }

        loop {
            rest = multispace0(rest)?.0;
            if let Some(after_comma) = rest.strip_prefix(',') {
                let after_ws = multispace0(after_comma)?.0;
                if after_ws.starts_with(',') || after_ws.starts_with(')') {
                    // Empty argument
                    args.push(Expr::Variable(String::new(), Span::new(0, 0)));
                    rest = after_comma;
                } else {
                    // Parse from the first non-whitespace token, not from
                    // just after the comma — a compound (BinaryOp) argument's
                    // span is measured from its entry point here, so passing
                    // the untrimmed `after_comma` would make the span start
                    // at the separating whitespace instead of the argument's
                    // own first token.
                    let (r, arg) = self.parse_comparison(after_ws)?;
                    args.push(arg);
                    rest = r;
                }
            } else {
                break;
            }
        }

        Ok((rest, args))
    }

    fn parse_array_elements(&self, i: &'a str) -> IResult<&'a str, Vec<Expr>> {
        let mut rows: Vec<Vec<Expr>> = Vec::new();
        let mut current_row: Vec<Expr> = Vec::new();
        let mut rest = multispace0(i)?.0;
        if rest.starts_with('}') {
            return Ok((rest, Vec::new())); // empty array {}
        }
        let (r, first) = self.parse_comparison(rest)?;
        current_row.push(first);
        rest = r;
        loop {
            rest = multispace0(rest)?.0;
            if let Some(after_comma) = rest.strip_prefix(',') {
                let (r, elem) = self.parse_comparison(after_comma)?;
                current_row.push(elem);
                rest = r;
            } else if let Some(after_semi) = rest.strip_prefix(';') {
                rows.push(std::mem::take(&mut current_row));
                let (r, elem) = self.parse_comparison(after_semi)?;
                current_row.push(elem);
                rest = r;
            } else {
                break;
            }
        }
        rows.push(current_row);
        // If only one row (no semicolons), return flat vec
        if rows.len() == 1 {
            return Ok((rest, rows.into_iter().next().unwrap()));
        }
        // Multiple rows → wrap each row in an Array node
        let span_start = i;
        let row_exprs: Vec<Expr> = rows
            .into_iter()
            .map(|row_elems| {
                let s = self.span(span_start, rest);
                Expr::Array(row_elems, s)
            })
            .collect();
        Ok((rest, row_exprs))
    }

    // ── postfix % ─────────────────────────────────────────────────────────

    fn parse_postfix(&self, i: &'a str) -> IResult<&'a str, Expr> {
        let (rest, expr) = self.parse_primary(i)?;
        let rest_ws = multispace0(rest)?.0;
        if let Some(after) = rest_ws.strip_prefix('%') {
            return Ok((after, Expr::UnaryOp {
                op: UnaryOp::Percent,
                operand: Box::new(expr),
                span: self.span(i, after),
            }));
        }
        Ok((rest, expr))
    }

    // ── unary minus ───────────────────────────────────────────────────────

    fn parse_unary(&self, i: &'a str) -> IResult<&'a str, Expr> {
        let i_ws = multispace0(i)?.0;
        if let Some(after_minus) = i_ws.strip_prefix('-') {
            let (rest, operand) = self.parse_unary(after_minus)?;
            return Ok((rest, Expr::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
                span: self.span(i_ws, rest),
            }));
        }
        self.parse_postfix(i)
    }

    // ── power ^ (right-associative) ───────────────────────────────────────

    fn parse_power(&self, i: &'a str) -> IResult<&'a str, Expr> {
        let (rest, left) = self.parse_unary(i)?;
        let rest_ws = multispace0(rest)?.0;
        if let Some(after_op) = rest_ws.strip_prefix('^') {
            let (rest2, right) = self.parse_power(after_op)?;
            return Ok((rest2, Expr::BinaryOp {
                op: BinaryOp::Pow,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(i, rest2),
            }));
        }
        Ok((rest, left))
    }

    // ── multiplicative * / ────────────────────────────────────────────────

    fn parse_multiplicative(&self, i: &'a str) -> IResult<&'a str, Expr> {
        let (mut rest, mut left) = self.parse_power(i)?;
        loop {
            let ws = multispace0(rest)?.0;
            let op = ws.strip_prefix('*').map(|after| (BinaryOp::Mul, after))
                .or_else(|| ws.strip_prefix('/').map(|after| (BinaryOp::Div, after)));
            match op {
                None => break,
                Some((op, after)) => {
                    let (r, right) = self.parse_power(after)?;
                    left = Expr::BinaryOp {
                        op,
                        span: self.span(i, r),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                    rest = r;
                }
            }
        }
        Ok((rest, left))
    }

    // ── additive + - ──────────────────────────────────────────────────────

    fn parse_additive(&self, i: &'a str) -> IResult<&'a str, Expr> {
        let (mut rest, mut left) = self.parse_multiplicative(i)?;
        loop {
            let ws = multispace0(rest)?.0;
            let op = ws.strip_prefix('+').map(|after| (BinaryOp::Add, after))
                .or_else(|| ws.strip_prefix('-').map(|after| (BinaryOp::Sub, after)));
            match op {
                None => break,
                Some((op, after)) => {
                    let (r, right) = self.parse_multiplicative(after)?;
                    left = Expr::BinaryOp {
                        op,
                        span: self.span(i, r),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                    rest = r;
                }
            }
        }
        Ok((rest, left))
    }

    // ── concat & ─────────────────────────────────────────────────────────

    fn parse_concat(&self, i: &'a str) -> IResult<&'a str, Expr> {
        let (mut rest, mut left) = self.parse_additive(i)?;
        loop {
            let ws = multispace0(rest)?.0;
            if let Some(after) = ws.strip_prefix('&') {
                let (r, right) = self.parse_additive(after)?;
                left = Expr::BinaryOp {
                    op: BinaryOp::Concat,
                    span: self.span(i, r),
                    left: Box::new(left),
                    right: Box::new(right),
                };
                rest = r;
            } else {
                break;
            }
        }
        Ok((rest, left))
    }

    // ── comparison = <> < > <= >= ─────────────────────────────────────────

    fn parse_comparison(&self, i: &'a str) -> IResult<&'a str, Expr> {
        let (rest, left) = self.parse_concat(i)?;
        let ws = multispace0(rest)?.0;

        // Longest match first
        let op_result: Option<(BinaryOp, &'a str)> = if let Some(after) = ws.strip_prefix("<>") {
            Some((BinaryOp::Ne, after))
        } else if let Some(after) = ws.strip_prefix("<=") {
            Some((BinaryOp::Le, after))
        } else if let Some(after) = ws.strip_prefix(">=") {
            Some((BinaryOp::Ge, after))
        } else if let Some(after) = ws.strip_prefix('<') {
            Some((BinaryOp::Lt, after))
        } else if let Some(after) = ws.strip_prefix('>') {
            Some((BinaryOp::Gt, after))
        } else if let Some(after) = ws.strip_prefix('=') {
            Some((BinaryOp::Eq, after))
        } else {
            None
        };

        if let Some((op, after)) = op_result {
            let (r, right) = self.parse_concat(after)?;
            return Ok((r, Expr::BinaryOp {
                op,
                span: self.span(i, r),
                left: Box::new(left),
                right: Box::new(right),
            }));
        }

        Ok((rest, left))
    }
}

// ── public API ──────────────────────────────────────────────────────────────

/// Parse a formula string into an expression tree.
///
/// The formula must start with `=`. Returns a [`ParseError`] if the input
/// is not a valid formula.
#[deprecated(since = "0.7.0", note = "use Engine::sheets()/Engine::excel() and engine.parse() — engine flavor is required; see ADR 2026-04-27; removal target: 0.7.0 coordinated release")]
pub fn parse(formula: &str) -> Result<Expr, ParseError> {
    parse_formula(formula)
}

/// Crate-internal parser entry point used by [`crate::Engine`].
pub(crate) fn parse_formula(formula: &str) -> Result<Expr, ParseError> {
    let input = formula.strip_prefix('=').unwrap_or(formula).trim();
    let p = Parser::new(formula);
    match p.parse_comparison(input) {
        Ok((rest, expr)) => {
            let rest = rest.trim();
            if rest.is_empty() {
                Ok(expr)
            } else {
                Err(ParseError {
                    message: format!("Unexpected input '{}'", rest),
                    position: offset(formula, rest),
                })
            }
        }
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => Err(ParseError {
            message: "Parse error".into(),
            position: offset(formula, e.input),
        }),
        Err(nom::Err::Incomplete(_)) => Err(ParseError {
            message: "Incomplete input".into(),
            position: formula.len(),
        }),
    }
}

/// Validate that a formula string is syntactically correct without returning the AST.
#[deprecated(since = "0.7.0", note = "use Engine::sheets()/Engine::excel() and engine.validate() — engine flavor is required; see ADR 2026-04-27; removal target: 0.7.0 coordinated release")]
pub fn validate(formula: &str) -> Result<(), ParseError> {
    parse_formula(formula).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::parse_formula as parse;
    use crate::parser::ast::{BinaryOp, Expr, UnaryOp};

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
}
