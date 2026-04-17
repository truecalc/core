use super::super::sheets_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::Value;

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    sheets_fn(&args, &mut ctx)
}

#[test]
fn no_args_returns_one() {
    assert_eq!(run(vec![]), Value::Number(1.0));
}

#[test]
fn ref_arg_returns_one() {
    let args = vec![Expr::Variable("A1".to_string(), span())];
    assert_eq!(run(args), Value::Number(1.0));
}
