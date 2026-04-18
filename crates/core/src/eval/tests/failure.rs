use super::super::*;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{BinaryOp, Expr, Span, UnaryOp};
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
fn unary_neg_text_error() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::UnaryOp {
        op: UnaryOp::Neg,
        operand: Box::new(Expr::Text("abc".to_string(), dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Error(ErrorKind::Value));
}

#[test]
fn binary_div_by_zero() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Div,
        left: Box::new(Expr::Number(10.0, dummy_span())),
        right: Box::new(Expr::Number(0.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn error_propagates_left_in_add() {
    let (reg, ctx) = make_ctx();
    // "abc" + 1 => #VALUE! (left error from to_number)
    let expr = Expr::BinaryOp {
        op: BinaryOp::Add,
        left: Box::new(Expr::Text("abc".to_string(), dummy_span())),
        right: Box::new(Expr::Number(1.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Error(ErrorKind::Value));
}

#[test]
fn error_propagates_left_over_right_in_cmp() {
    let (reg, _) = make_ctx();
    let mut vars = std::collections::HashMap::new();
    vars.insert("L".to_string(), Value::Error(ErrorKind::DivByZero));
    vars.insert("R".to_string(), Value::Error(ErrorKind::Ref));
    let ctx = Context::new(vars);
    let expr = Expr::BinaryOp {
        op: BinaryOp::Eq,
        left: Box::new(Expr::Variable("L".to_string(), dummy_span())),
        right: Box::new(Expr::Variable("R".to_string(), dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn function_eager_propagates_arg_error() {
    let mut reg = Registry::new();
    reg.register_internal("IDENTITY", |args| args.first().cloned().unwrap_or(Value::Empty));
    let mut vars = std::collections::HashMap::new();
    vars.insert("E".to_string(), Value::Error(ErrorKind::Ref));
    let ctx = Context::new(vars);
    let expr = Expr::FunctionCall {
        name: "IDENTITY".to_string(),
        args: vec![Expr::Variable("E".to_string(), dummy_span())],
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Error(ErrorKind::Ref));
}
