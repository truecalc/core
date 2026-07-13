use super::super::*;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{BinaryOp, Expr, Span};
use crate::types::{ErrorKind, Value};

fn dummy_span() -> Span { Span::new(0, 1) }

fn make_ctx() -> (Registry, Context) {
    (Registry::new(), Context::empty())
}

fn run(expr: &Expr, registry: &Registry, ctx: Context) -> Value {
    let mut eval_ctx = EvalCtx::new(ctx, registry);
    evaluate_expr(expr, &mut eval_ctx)
}

#[test]
fn variable_not_found() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::Variable("missing".to_string(), dummy_span());
    assert_eq!(run(&expr, &reg, ctx), Value::Empty);
}

#[test]
fn bare_variable_override_lookup_is_dollar_insensitive() {
    // A caller-supplied override bound under the plain key must still be
    // found when the formula reads it via a $-anchored bare reference --
    // $ is a display marker only (issue #708).
    let (reg, mut ctx) = make_ctx();
    ctx.set("A1".to_string(), Value::Number(42.0));
    let expr = Expr::Variable("$A$1".to_string(), dummy_span());
    assert_eq!(run(&expr, &reg, ctx), Value::Number(42.0));
}

#[test]
fn cmp_eq_cross_type() {
    // Number vs Text: always false for Eq
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Eq,
        left: Box::new(Expr::Number(1.0, dummy_span())),
        right: Box::new(Expr::Text("1".to_string(), dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Bool(false));
}

#[test]
fn cmp_ne_cross_type() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Ne,
        left: Box::new(Expr::Number(1.0, dummy_span())),
        right: Box::new(Expr::Text("1".to_string(), dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Bool(true));
}

#[test]
fn cmp_lt_number_vs_text() {
    // Number < Text in Excel ordering
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Lt,
        left: Box::new(Expr::Number(999.0, dummy_span())),
        right: Box::new(Expr::Text("a".to_string(), dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Bool(true));
}

#[test]
fn cmp_bool_ordering() {
    // FALSE < TRUE
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Lt,
        left: Box::new(Expr::Bool(false, dummy_span())),
        right: Box::new(Expr::Bool(true, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Bool(true));
}

#[test]
fn cmp_text_gt_number() {
    // Text > Number in Excel ordering
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Gt,
        left: Box::new(Expr::Text("a".to_string(), dummy_span())),
        right: Box::new(Expr::Number(999.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Bool(true));
}

#[test]
fn pow_negative_fractional_exponent_is_num_error() {
    // (-8.0).powf(0.333) → NaN → #NUM!
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Pow,
        left: Box::new(Expr::Number(-8.0, dummy_span())),
        right: Box::new(Expr::Number(0.333, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Error(ErrorKind::Num));
}

#[test]
fn pow_overflow_to_infinity_is_num_error() {
    // 1e308.powf(2.0) → infinity → #NUM!
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Pow,
        left: Box::new(Expr::Number(1e308, dummy_span())),
        right: Box::new(Expr::Number(2.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Error(ErrorKind::Num));
}

#[test]
fn reference_override_lookup_is_dollar_insensitive() {
    // A caller-supplied override bound under the plain canonical key must
    // still be found when the formula reads the cell via a $-anchored
    // reference -- $ is a display marker only (issue #708).
    let (reg, mut ctx) = make_ctx();
    ctx.set("Sheet1!A1".to_string(), Value::Number(99.0));
    let r = crate::parser::refs::Ref::Cell {
        sheet: Some("Sheet1".to_string()),
        addr: crate::parser::refs::CellAddr::new(1, 1).with_col_abs(true).with_row_abs(true),
    };
    let expr = Expr::Reference(r, dummy_span());
    assert_eq!(run(&expr, &reg, ctx), Value::Number(99.0));
}

#[test]
fn reference_override_lookup_does_not_corrupt_dollar_sheet_name() {
    // A sheet literally named "Q1$Data" contains '$' as a real character,
    // not a reference anchor. The override-lookup key must only strip $
    // from the CellAddr portion (built structurally via relative_display,
    // never by string-replacing the whole rendered text) so the sheet
    // name's own '$' survives untouched (issue #708).
    let (reg, mut ctx) = make_ctx();
    ctx.set("'Q1$Data'!A1".to_string(), Value::Number(7.0));
    let r = crate::parser::refs::Ref::Cell {
        sheet: Some("Q1$Data".to_string()),
        addr: crate::parser::refs::CellAddr::new(1, 1).with_col_abs(true),
    };
    let expr = Expr::Reference(r, dummy_span());
    assert_eq!(run(&expr, &reg, ctx), Value::Number(7.0));
}
