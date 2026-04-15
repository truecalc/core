use super::super::*;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::{ErrorKind, Value};

fn dummy_span() -> Span {
    Span::new(0, 1)
}

fn make_eval_ctx(reg: &Registry) -> EvalCtx<'_> {
    EvalCtx::new(Context::empty(), reg)
}

#[test]
fn text_condition_returns_value_error() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![
        Expr::Text("text".to_string(), dummy_span()),
        Expr::Number(1.0, dummy_span()),
        Expr::Number(2.0, dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Error(ErrorKind::Value));
}

#[test]
fn error_condition_propagates() {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(
        Context::new({
            let mut m = std::collections::HashMap::new();
            m.insert("E".to_string(), Value::Error(ErrorKind::Ref));
            m
        }),
        &reg,
    );
    let args = vec![
        Expr::Variable("E".to_string(), dummy_span()),
        Expr::Number(1.0, dummy_span()),
        Expr::Number(2.0, dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Error(ErrorKind::Ref));
}

#[test]
fn zero_args_returns_value_error() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    assert_eq!(if_fn(&[], &mut ctx), Value::Error(ErrorKind::Value));
}

#[test]
fn one_arg_returns_value_error() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![Expr::Bool(true, dummy_span())];
    assert_eq!(if_fn(&args, &mut ctx), Value::Error(ErrorKind::Value));
}

#[test]
fn four_args_returns_value_error() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![
        Expr::Bool(true, dummy_span()),
        Expr::Number(1.0, dummy_span()),
        Expr::Number(2.0, dummy_span()),
        Expr::Number(3.0, dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Error(ErrorKind::Value));
}
