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
fn if_true_returns_then() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![
        Expr::Bool(true, dummy_span()),
        Expr::Number(1.0, dummy_span()),
        Expr::Number(2.0, dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Number(1.0));
}

#[test]
fn if_false_returns_else() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![
        Expr::Bool(false, dummy_span()),
        Expr::Number(1.0, dummy_span()),
        Expr::Number(2.0, dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Number(2.0));
}

#[test]
fn nonzero_number_is_truthy() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![
        Expr::Number(1.0, dummy_span()),
        Expr::Text("yes".to_string(), dummy_span()),
        Expr::Text("no".to_string(), dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Text("yes".to_string()));
}

#[test]
fn zero_number_is_falsy() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![
        Expr::Number(0.0, dummy_span()),
        Expr::Text("yes".to_string(), dummy_span()),
        Expr::Text("no".to_string(), dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Text("no".to_string()));
}

#[test]
fn two_arg_form_condition_true() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![
        Expr::Bool(true, dummy_span()),
        Expr::Number(1.0, dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Number(1.0));
}

#[test]
fn two_arg_form_condition_false_returns_bool_false() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![
        Expr::Bool(false, dummy_span()),
        Expr::Number(1.0, dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Bool(false));
}
