pub mod ast;
pub mod tokens;

pub use ast::Expr;
use ast::{BinaryOp, Span, UnaryOp};
use crate::types::ParseError;
use nom::{IResult, character::complete::multispace0};
use tokens::{bool_literal, identifier, number_literal, offset, string_literal};

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

        // Identifier: variable or function call
        if let Ok((rest, name)) = identifier(i) {
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
            // Range reference: A1:D4
            if is_cell_ref(name) {
                if let Some(after_colon) = rest_ws.strip_prefix(':') {
                    if let Ok((rest2, name2)) = identifier(after_colon) {
                        if is_cell_ref(name2) {
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
                    let (r, arg) = self.parse_comparison(after_comma)?;
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

/// Returns true if `name` looks like a cell reference (e.g. "A1", "BC42").
fn is_cell_ref(name: &str) -> bool {
    let bytes = name.as_bytes();
    let col_end = bytes.iter().take_while(|b| b.is_ascii_alphabetic()).count();
    col_end > 0 && col_end < bytes.len() && bytes[col_end..].iter().all(|b| b.is_ascii_digit())
}

// ── public API ──────────────────────────────────────────────────────────────

/// Parse a formula string into an expression tree.
///
/// The formula must start with `=`. Returns a [`ParseError`] if the input
/// is not a valid formula.
pub fn parse(formula: &str) -> Result<Expr, ParseError> {
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
pub fn validate(formula: &str) -> Result<(), ParseError> {
    parse(formula).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
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
