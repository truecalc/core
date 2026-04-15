use super::super::{iferror_fn, ifna_fn};
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
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

/// IFERROR catches ALL error kinds including NA.
#[test]
fn iferror_catches_na() {
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
        Expr::Bool(false, span()),
    ];
    assert_eq!(iferror_fn(&args, &mut ctx), Value::Bool(false));
}

/// IFNA does NOT catch non-NA errors — it passes them through.
#[test]
fn ifna_passes_through_non_na_error() {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(
        Context::new({
            let mut m = std::collections::HashMap::new();
            m.insert("X".to_string(), Value::Error(ErrorKind::DivByZero));
            m
        }),
        &reg,
    );
    let args = vec![
        Expr::Variable("X".to_string(), span()),
        Expr::Number(0.0, span()),
    ];
    assert_eq!(ifna_fn(&args, &mut ctx), Value::Error(ErrorKind::DivByZero));
}

/// IFNA does NOT catch Value errors.
#[test]
fn ifna_passes_through_value_error() {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(
        Context::new({
            let mut m = std::collections::HashMap::new();
            m.insert("X".to_string(), Value::Error(ErrorKind::Value));
            m
        }),
        &reg,
    );
    let args = vec![
        Expr::Variable("X".to_string(), span()),
        Expr::Number(0.0, span()),
    ];
    assert_eq!(ifna_fn(&args, &mut ctx), Value::Error(ErrorKind::Value));
}
