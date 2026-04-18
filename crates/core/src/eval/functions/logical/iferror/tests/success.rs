use super::super::{iferror_fn, ifna_fn};
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{BinaryOp, Expr, Span};
use crate::types::{ErrorKind, Value};

fn span() -> Span { Span::new(0, 1) }

fn run_iferror(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    iferror_fn(&args, &mut ctx)
}

fn run_ifna(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    ifna_fn(&args, &mut ctx)
}

#[test]
fn iferror_on_error_returns_fallback() {
    // IFERROR(1/0, "fallback") → "fallback"
    let args = vec![
        Expr::BinaryOp {
            op: BinaryOp::Div,
            left: Box::new(Expr::Number(1.0, span())),
            right: Box::new(Expr::Number(0.0, span())),
            span: span(),
        },
        Expr::Text("fallback".to_string(), span()),
    ];
    assert_eq!(run_iferror(args), Value::Text("fallback".to_string()));
}

#[test]
fn iferror_on_success_returns_value() {
    // IFERROR(5, "fallback") → 5
    let args = vec![
        Expr::Number(5.0, span()),
        Expr::Text("fallback".to_string(), span()),
    ];
    assert_eq!(run_iferror(args), Value::Number(5.0));
}

#[test]
fn ifna_on_na_returns_fallback() {
    // Trigger #N/A via a variable set to Value::Error(ErrorKind::NA)
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(
        Context::new({
            let mut m = std::collections::HashMap::new();
            m.insert("X".to_string(), Value::Error(ErrorKind::NA));
            m
        }),
        &reg,
    );
    let args = vec![
        Expr::Variable("X".to_string(), span()),
        Expr::Text("na_fallback".to_string(), span()),
    ];
    assert_eq!(ifna_fn(&args, &mut ctx), Value::Text("na_fallback".to_string()));
}

#[test]
fn ifna_on_other_error_returns_error() {
    // IFNA(1/0, "fallback") → #DIV/0! (IFNA does NOT catch it)
    let args = vec![
        Expr::BinaryOp {
            op: BinaryOp::Div,
            left: Box::new(Expr::Number(1.0, span())),
            right: Box::new(Expr::Number(0.0, span())),
            span: span(),
        },
        Expr::Text("fallback".to_string(), span()),
    ];
    assert_eq!(run_ifna(args), Value::Error(ErrorKind::DivByZero));
}
