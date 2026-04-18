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

#[test]
fn iferror_zero_args_returns_na() {
    assert_eq!(run_iferror(vec![]), Value::Error(ErrorKind::NA));
}

#[test]
fn iferror_too_many_args() {
    let args = vec![
        Expr::Number(1.0, span()),
        Expr::Number(2.0, span()),
        Expr::Number(3.0, span()),
    ];
    assert_eq!(run_iferror(args), Value::Error(ErrorKind::NA));
}

#[test]
fn ifna_zero_args_returns_na() {
    assert_eq!(run_ifna(vec![]), Value::Error(ErrorKind::NA));
}
