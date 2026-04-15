use super::super::and_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{BinaryOp, Expr, Span};
use crate::types::Value;

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    and_fn(&args, &mut ctx)
}

/// `AND(FALSE, 1/0)` must short-circuit — the division by zero is never evaluated.
#[test]
fn short_circuits_on_first_false() {
    let div_by_zero = Expr::BinaryOp {
        op: BinaryOp::Div,
        left: Box::new(Expr::Number(1.0, span())),
        right: Box::new(Expr::Number(0.0, span())),
        span: span(),
    };
    let args = vec![Expr::Bool(false, span()), div_by_zero];
    assert_eq!(run(args), Value::Bool(false));
}

#[test]
fn zero_is_falsy() {
    let args = vec![Expr::Number(0.0, span())];
    assert_eq!(run(args), Value::Bool(false));
}

#[test]
fn mixed_true_and_false() {
    let args = vec![Expr::Bool(true, span()), Expr::Number(0.0, span())];
    assert_eq!(run(args), Value::Bool(false));
}
