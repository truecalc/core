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

// ── Leaf nodes ───────────────────────────────────────────────────────────────

#[test]
fn leaf_number() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::Number(42.0, dummy_span());
    assert_eq!(run(&expr, &reg, ctx), Value::Number(42.0));
}

#[test]
fn leaf_text() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::Text("hello".to_string(), dummy_span());
    assert_eq!(run(&expr, &reg, ctx), Value::Text("hello".to_string()));
}

#[test]
fn leaf_bool_true() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::Bool(true, dummy_span());
    assert_eq!(run(&expr, &reg, ctx), Value::Bool(true));
}

#[test]
fn leaf_bool_false() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::Bool(false, dummy_span());
    assert_eq!(run(&expr, &reg, ctx), Value::Bool(false));
}

#[test]
fn variable_found() {
    let (reg, _) = make_ctx();
    let mut vars = std::collections::HashMap::new();
    vars.insert("x".to_string(), Value::Number(7.0));
    let ctx = Context::new(vars);
    let expr = Expr::Variable("x".to_string(), dummy_span());
    assert_eq!(run(&expr, &reg, ctx), Value::Number(7.0));
}

// ── Unary ops ────────────────────────────────────────────────────────────────

#[test]
fn unary_neg_number() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::UnaryOp {
        op: UnaryOp::Neg,
        operand: Box::new(Expr::Number(5.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Number(-5.0));
}

#[test]
fn unary_percent() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::UnaryOp {
        op: UnaryOp::Percent,
        operand: Box::new(Expr::Number(50.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Number(0.5));
}

// ── Binary arithmetic ────────────────────────────────────────────────────────

#[test]
fn binary_add() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Add,
        left: Box::new(Expr::Number(3.0, dummy_span())),
        right: Box::new(Expr::Number(4.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Number(7.0));
}

#[test]
fn binary_sub() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Sub,
        left: Box::new(Expr::Number(10.0, dummy_span())),
        right: Box::new(Expr::Number(3.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Number(7.0));
}

#[test]
fn binary_mul() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Mul,
        left: Box::new(Expr::Number(6.0, dummy_span())),
        right: Box::new(Expr::Number(7.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Number(42.0));
}

#[test]
fn binary_div() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Div,
        left: Box::new(Expr::Number(10.0, dummy_span())),
        right: Box::new(Expr::Number(4.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Number(2.5));
}

#[test]
fn binary_pow() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Pow,
        left: Box::new(Expr::Number(2.0, dummy_span())),
        right: Box::new(Expr::Number(10.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Number(1024.0));
}

// ── Concat ───────────────────────────────────────────────────────────────────

#[test]
fn binary_concat() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Concat,
        left: Box::new(Expr::Text("hello".to_string(), dummy_span())),
        right: Box::new(Expr::Text(" world".to_string(), dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Text("hello world".to_string()));
}

// ── Same-type comparisons ────────────────────────────────────────────────────

#[test]
fn cmp_eq_same_type_equal() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Eq,
        left: Box::new(Expr::Number(5.0, dummy_span())),
        right: Box::new(Expr::Number(5.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Bool(true));
}

#[test]
fn cmp_eq_same_type_unequal() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::BinaryOp {
        op: BinaryOp::Eq,
        left: Box::new(Expr::Number(5.0, dummy_span())),
        right: Box::new(Expr::Number(6.0, dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Bool(false));
}

// ── Function dispatch ────────────────────────────────────────────────────────

#[test]
fn function_unknown_returns_name_error() {
    let (reg, ctx) = make_ctx();
    let expr = Expr::FunctionCall {
        name: "NONEXISTENT".to_string(),
        args: vec![],
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Error(ErrorKind::Name));
}

#[test]
fn function_eager_with_registered_fn() {
    // Register a custom eager fn: DOUBLE(x) = x * 2
    let mut reg = Registry::new();
    reg.register_internal("DOUBLE", |args| {
        if let Some(Value::Number(n)) = args.first() {
            Value::Number(n * 2.0)
        } else {
            Value::Error(ErrorKind::Value)
        }
    });
    let ctx = Context::empty();
    let expr = Expr::FunctionCall {
        name: "DOUBLE".to_string(),
        args: vec![Expr::Number(21.0, dummy_span())],
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Number(42.0));
}

#[test]
fn function_lazy_receives_raw_args() {
    // Register a lazy fn that reads a variable from ctx — proves EvalCtx is passed correctly
    // and args are NOT pre-evaluated.
    let mut reg = Registry::new();
    reg.register_internal_lazy("LAZY_VAR", |args, ctx| {
        // The first arg is a Variable expr; evaluate it manually to prove ctx is live
        evaluate_expr(&args[0], ctx)
    });
    let mut vars = std::collections::HashMap::new();
    vars.insert("X".to_string(), Value::Number(99.0));
    let ctx = Context::new(vars);
    let expr = Expr::FunctionCall {
        name: "LAZY_VAR".to_string(),
        args: vec![Expr::Variable("X".to_string(), dummy_span())],
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Number(99.0));
}

#[test]
fn function_eager_first_error_wins_over_second() {
    let mut reg = Registry::new();
    reg.register_internal("ADD2", |args| {
        match (args.get(0), args.get(1)) {
            (Some(Value::Number(a)), Some(Value::Number(b))) => Value::Number(a + b),
            _ => Value::Error(ErrorKind::Value),
        }
    });
    // arg[0] = #DIV/0!, arg[1] = #REF! → first error should win
    let mut vars = std::collections::HashMap::new();
    vars.insert("A".to_string(), Value::Error(ErrorKind::DivByZero));
    vars.insert("B".to_string(), Value::Error(ErrorKind::Ref));
    let ctx = Context::new(vars);
    let expr = Expr::FunctionCall {
        name: "ADD2".to_string(),
        args: vec![
            Expr::Variable("A".to_string(), dummy_span()),
            Expr::Variable("B".to_string(), dummy_span()),
        ],
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn concat_right_error_propagates() {
    let (reg, _) = make_ctx();
    let mut vars = std::collections::HashMap::new();
    vars.insert("E".to_string(), Value::Error(ErrorKind::Value));
    let ctx = Context::new(vars);
    let expr = Expr::BinaryOp {
        op: BinaryOp::Concat,
        left: Box::new(Expr::Text("hello".to_string(), dummy_span())),
        right: Box::new(Expr::Variable("E".to_string(), dummy_span())),
        span: dummy_span(),
    };
    assert_eq!(run(&expr, &reg, ctx), Value::Error(ErrorKind::Value));
}
