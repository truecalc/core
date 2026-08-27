pub mod ast;
pub mod refs;
pub mod tokens;

pub use ast::{Expr, Span};
pub use refs::{CellAddr, Ref};
use ast::{BinaryOp, UnaryOp};
use crate::types::ParseError;
use nom::{IResult, character::complete::multispace0};
use tokens::{bool_literal, dollar_cell_ref, error_literal, identifier, number_literal, offset, string_literal};

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
            // Trim whitespace after '(' so a padded grouping like `( A1 + B1 )`
            // does not leak the leading space into the inner expression's span
            // (issue #751 — same class as #746/#748/#749, which also trim before
            // parse_comparison; the trailing side is already trimmed below).
            let inner = multispace0(inner)?.0;
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

        // Error literal: #REF!, #DIV/0!, #NAME?, #VALUE!, #NUM!, #N/A, #NULL!
        // (issue #716) — parses straight to its error value, same as a
        // number/string/boolean literal parses straight to theirs. No other
        // primary form starts with '#', so this can be tried unconditionally.
        if let Ok((rest, kind)) = error_literal(i) {
            return Ok((rest, Expr::Error(kind, self.span(i, rest))));
        }

        // Unqualified current-row table reference: [@Column]. Only the `@`
        // form is legal unqualified — a bare `[Column]` names no table and
        // is a parse error (Task 3 test `bracket_without_at_is_a_parse_error`).
        if let Some(after_bracket) = i.strip_prefix('[') {
            if after_bracket.starts_with('@') {
                return self.parse_table_ref(i, None, after_bracket);
            }
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
            // Table reference: Table[Column] or Table[@Column] — `[` binds
            // tightly to the table name (no whitespace on either side).
            if let Some(after_bracket) = rest.strip_prefix('[') {
                return self.parse_table_ref(i, Some(name.to_string()), after_bracket);
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

    /// Parse the part after `[` in a table reference: an optional `@`, a
    /// column identifier, then `]`. `start` is where the whole reference
    /// began (for spans); `table` is `None` for the unqualified `[@Column]`
    /// form (called from `parse_primary`'s top-level `[` check, Task 3).
    fn parse_table_ref(
        &self,
        start: &'a str,
        table: Option<String>,
        i: &'a str,
    ) -> IResult<&'a str, Expr> {
        let err = || nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Tag));
        let this_row = i.starts_with('@');
        let i = if this_row { &i[1..] } else { i };
        let (rest, column) = identifier(i).map_err(|_| err())?;
        let after = rest.strip_prefix(']').ok_or_else(err)?;
        let r = Ref::Table { table, column: column.to_string(), this_row };
        Ok((after, Expr::Reference(r, self.span(start, after))))
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
                // Parse from the first non-whitespace token, not from just
                // after the comma — same leading-whitespace bug as #746's
                // function-argument fix, but here in the array-element
                // separator: passing the untrimmed `after_comma` would let a
                // compound (BinaryOp) element's span start at the separating
                // whitespace instead of the element's own first token.
                let after_ws = multispace0(after_comma)?.0;
                let (r, elem) = self.parse_comparison(after_ws)?;
                current_row.push(elem);
                rest = r;
            } else if let Some(after_semi) = rest.strip_prefix(';') {
                rows.push(std::mem::take(&mut current_row));
                // Same trim as the comma branch above, for the first element
                // of the new row.
                let after_ws = multispace0(after_semi)?.0;
                let (r, elem) = self.parse_comparison(after_ws)?;
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
        // Multiple rows → wrap each row in an Array node. Each row's span
        // must cover only that row's own elements (its first element's start
        // to its last element's end) — not the whole `{…}` body, which is
        // what every row got when this span was computed once outside the
        // loop below.
        let row_exprs: Vec<Expr> = rows
            .into_iter()
            .map(|row_elems| {
                let s = match (row_elems.first(), row_elems.last()) {
                    (Some(first), Some(last)) => {
                        let start = first.span().offset;
                        let end = last.span().offset + last.span().length;
                        Span::new(start, end - start)
                    }
                    // A row is never empty in practice (each row starts with
                    // an element pushed either before the loop or right
                    // after a `;`), but fall back to the old whole-body span
                    // rather than panic if that ever changes.
                    _ => self.span(i, rest),
                };
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
#[deprecated(since = "0.7.0", note = "use parse_formula() instead — parsing is flavor-independent, so no Engine is required; see ADR 2026-04-27; removal target: 0.7.0 coordinated release")]
pub fn parse(formula: &str) -> Result<Expr, ParseError> {
    parse_formula(formula)
}

/// Parse a formula string into an expression tree, without an [`Engine`].
///
/// This is the parser entry point [`Engine::parse`] and [`Engine::validate`]
/// call. It is exposed directly because parsing is **flavor-independent and
/// registry-free**: it reads only the formula text, so a caller that needs an
/// AST (or only a syntax check) never has to construct an [`Engine`] — and
/// therefore never has to build the function [`Registry`], which parsing does
/// not consult (issue #900). Building that registry costs orders of magnitude
/// more than the parse it was being built for.
///
/// The leading `=` is optional.
///
/// This is not a reversal of the flavor-explicit direction taken for
/// evaluation (see ADR 2026-04-27) — parsing and evaluation are different
/// operations. Evaluation requires an engine flavor because function
/// behavior can differ across flavors; parsing does not, and this is
/// verified rather than assumed: the parser holds no reference to
/// [`Registry`] or [`Engine`], and an unknown function name fails at
/// evaluation time, not at parse time.
///
/// [`Engine`]: crate::Engine
/// [`Engine::parse`]: crate::Engine::parse
/// [`Engine::validate`]: crate::Engine::validate
/// [`Registry`]: crate::Registry
pub fn parse_formula(formula: &str) -> Result<Expr, ParseError> {
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
mod tests;
