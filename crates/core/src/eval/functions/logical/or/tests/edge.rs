use super::super::or_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{BinaryOp, Expr, Span};
use crate::types::Value;

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    or_fn(&args, &mut ctx)
}

/// `OR(TRUE, 1/0)` must short-circuit — the division by zero is never evaluated.
#[test]
fn short_circuits_on_first_true() {
    let div_by_zero = Expr::BinaryOp {
        op: BinaryOp::Div,
        left: Box::new(Expr::Number(1.0, span())),
        right: Box::new(Expr::Number(0.0, span())),
        span: span(),
    };
    let args = vec![Expr::Bool(true, span()), div_by_zero];
    assert_eq!(run(args), Value::Bool(true));
}

#[test]
fn zero_is_falsy() {
    let args = vec![Expr::Number(0.0, span())];
    assert_eq!(run(args), Value::Bool(false));
}

#[test]
fn single_false() {
    let args = vec![Expr::Bool(false, span())];
    assert_eq!(run(args), Value::Bool(false));
}

#[test]
fn array_with_one_true_element() {
    // OR({FALSE,FALSE,TRUE}) = TRUE — array is flattened, TRUE element found
    let result = crate::evaluate("=OR({FALSE,FALSE,TRUE})", &std::collections::HashMap::new());
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn array_all_false() {
    // OR({FALSE,FALSE,FALSE}) = FALSE — no truthy elements
    let result = crate::evaluate("=OR({FALSE,FALSE,FALSE})", &std::collections::HashMap::new());
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn array_with_nonzero() {
    // OR({0,0,2}) = TRUE — nonzero number is truthy
    let result = crate::evaluate("=OR({0,0,2})", &std::collections::HashMap::new());
    assert_eq!(result, Value::Bool(true));
}
