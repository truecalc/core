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
fn two_arg_form_condition_false_returns_bool_false() {
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![
        Expr::Bool(false, dummy_span()),
        Expr::Number(1.0, dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Bool(false));
}

#[test]
fn short_circuit_true_does_not_evaluate_else() {
    // IF(TRUE, 1, NONEXISTENT()) → 1
    // If NONEXISTENT were evaluated it would produce #NAME!; short-circuit skips it.
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let args = vec![
        Expr::Bool(true, dummy_span()),
        Expr::Number(1.0, dummy_span()),
        Expr::FunctionCall {
            name: "NONEXISTENT".to_string(),
            args: vec![],
            span: dummy_span(),
        },
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Number(1.0));
}

#[test]
fn nested_if() {
    // IF(TRUE, IF(FALSE, 1, 2), 3) → 2
    let reg = Registry::new();
    let mut ctx = make_eval_ctx(&reg);
    let inner_if = Expr::FunctionCall {
        name: "IF".to_string(),
        args: vec![
            Expr::Bool(false, dummy_span()),
            Expr::Number(1.0, dummy_span()),
            Expr::Number(2.0, dummy_span()),
        ],
        span: dummy_span(),
    };
    let args = vec![
        Expr::Bool(true, dummy_span()),
        inner_if,
        Expr::Number(3.0, dummy_span()),
    ];
    assert_eq!(if_fn(&args, &mut ctx), Value::Number(2.0));
}
