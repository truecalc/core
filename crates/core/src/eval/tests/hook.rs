//! Unit tests for the opt-in per-node evaluation hook (issue #732).
//!
//! These prove three properties of [`crate::eval::EvalHook`]:
//! 1. it fires exactly once per evaluated node, in post-order, carrying the
//!    correct operation and resulting value;
//! 2. lazy short-circuiting means only *evaluated* nodes fire (an unvisited
//!    `IF` branch never reports);
//! 3. wiring a hook (`Some`) does not change the computed value versus `None`.

use crate::eval::{Context, EvalCtx, EvalOp, Registry};
use crate::engine::Engine;
use crate::types::Value;

/// An owned, comparable snapshot of one hook event — `EvalOp` borrows from the
/// AST, so we flatten it into owned data the moment the callback fires.
#[derive(Debug, Clone, PartialEq)]
enum Op {
    Number,
    Text,
    Bool,
    Variable(String),
    Reference,
    UnaryOp,
    BinaryOp,
    Array,
    Apply,
    FunctionCall(String),
}

impl Op {
    fn from(op: EvalOp<'_>) -> Self {
        match op {
            EvalOp::Number => Op::Number,
            EvalOp::Text => Op::Text,
            EvalOp::Bool => Op::Bool,
            EvalOp::Variable(name) => Op::Variable(name.to_string()),
            EvalOp::Reference => Op::Reference,
            EvalOp::UnaryOp(_) => Op::UnaryOp,
            EvalOp::BinaryOp(_) => Op::BinaryOp,
            EvalOp::Array => Op::Array,
            EvalOp::Apply => Op::Apply,
            EvalOp::FunctionCall(name) => Op::FunctionCall(name.to_string()),
        }
    }
}

/// Evaluate `formula` with a recording hook wired, returning the final value
/// and the ordered list of `(operation, resulting value)` events.
fn trace(formula: &str) -> (Value, Vec<(Op, Value)>) {
    let engine = Engine::sheets();
    let expr = engine.parse(formula).expect("valid formula");
    let registry = Registry::new();

    let mut events: Vec<(Op, Value)> = Vec::new();
    let value = {
        let mut record = |op: EvalOp<'_>, v: &Value| events.push((Op::from(op), v.clone()));
        let mut ctx = EvalCtx::new(Context::empty(), &registry);
        ctx.hook = Some(&mut record);
        crate::eval::evaluate_expr(&expr, &mut ctx)
    };
    (value, events)
}

/// Evaluate `formula` with no hook wired.
fn eval_no_hook(formula: &str) -> Value {
    let engine = Engine::sheets();
    let expr = engine.parse(formula).expect("valid formula");
    let registry = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &registry);
    crate::eval::evaluate_expr(&expr, &mut ctx)
}

#[test]
fn fires_post_order_for_binary_op() {
    // =1+2 → leaves fire before the operator; each event carries its value.
    let (value, events) = trace("=1+2");
    assert_eq!(value, Value::Number(3.0));
    assert_eq!(
        events,
        vec![
            (Op::Number, Value::Number(1.0)),
            (Op::Number, Value::Number(2.0)),
            (Op::BinaryOp, Value::Number(3.0)),
        ]
    );
}

#[test]
fn fires_once_per_node_for_function_call() {
    // =SUM(1,2) → both arguments fire (post-order), then the call node.
    let (value, events) = trace("=SUM(1,2)");
    assert_eq!(value, Value::Number(3.0));
    assert_eq!(
        events,
        vec![
            (Op::Number, Value::Number(1.0)),
            (Op::Number, Value::Number(2.0)),
            (Op::FunctionCall("SUM".to_string()), Value::Number(3.0)),
        ]
    );
}

#[test]
fn nested_tree_fires_every_node_exactly_once() {
    // =(1+2)*3 → 5 nodes: 1, 2, (1+2), 3, ((1+2)*3).
    let (value, events) = trace("=(1+2)*3");
    assert_eq!(value, Value::Number(9.0));
    assert_eq!(
        events,
        vec![
            (Op::Number, Value::Number(1.0)),
            (Op::Number, Value::Number(2.0)),
            (Op::BinaryOp, Value::Number(3.0)),
            (Op::Number, Value::Number(3.0)),
            (Op::BinaryOp, Value::Number(9.0)),
        ]
    );
}

#[test]
fn lazy_branch_not_taken_never_fires() {
    // =IF(TRUE, 10, 20) → only the condition, the taken branch, and the call
    // fire; the untaken `20` literal is never evaluated, so never reported.
    let (value, events) = trace("=IF(TRUE, 10, 20)");
    assert_eq!(value, Value::Number(10.0));
    assert_eq!(
        events,
        vec![
            (Op::Bool, Value::Bool(true)),
            (Op::Number, Value::Number(10.0)),
            (Op::FunctionCall("IF".to_string()), Value::Number(10.0)),
        ]
    );
    // The value 20.0 must appear nowhere in the event stream.
    assert!(!events.iter().any(|(_, v)| *v == Value::Number(20.0)));
}

#[test]
fn unary_op_reports_operator_node() {
    let (value, events) = trace("=-5");
    assert_eq!(value, Value::Number(-5.0));
    assert_eq!(
        events,
        vec![
            (Op::Number, Value::Number(5.0)),
            (Op::UnaryOp, Value::Number(-5.0)),
        ]
    );
}

#[test]
fn text_and_concat_report_correct_ops() {
    let (value, events) = trace("=\"a\"&\"b\"");
    assert_eq!(value, Value::Text("ab".to_string()));
    assert_eq!(
        events,
        vec![
            (Op::Text, Value::Text("a".to_string())),
            (Op::Text, Value::Text("b".to_string())),
            (Op::BinaryOp, Value::Text("ab".to_string())),
        ]
    );
}

#[test]
fn array_literal_reports_elements_then_array() {
    let (value, events) = trace("={1,2}");
    assert_eq!(
        value,
        Value::Array(vec![Value::Number(1.0), Value::Number(2.0)])
    );
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].0, Op::Number);
    assert_eq!(events[1].0, Op::Number);
    assert_eq!(events[2].0, Op::Array);
}

#[test]
fn error_node_still_fires_with_error_value() {
    // Argument-error short-circuit inside a function still reports the call
    // node — the hook observes the resulting error value.
    let (value, events) = trace("=SQRT(-1)");
    assert!(value.is_error());
    // Last event is the function-call node carrying the error result.
    let (last_op, last_val) = events.last().expect("at least one event");
    assert_eq!(*last_op, Op::FunctionCall("SQRT".to_string()));
    assert!(last_val.is_error());
}

#[test]
fn none_hook_matches_some_hook_value() {
    // The hook is purely observational: the computed value is identical whether
    // a hook is wired or not, across a spread of expression shapes.
    for formula in [
        "=1+2",
        "=(1+2)*3",
        "=SUM(1,2,3)",
        "=IF(FALSE, 1, 2)",
        "=-5%",
        "=\"x\"&\"y\"",
        "={1,2,3}",
        "=AVERAGE(2,4,6)",
    ] {
        let (with_hook, _) = trace(formula);
        let without_hook = eval_no_hook(formula);
        assert_eq!(with_hook, without_hook, "value diverged for {formula}");
    }
}
