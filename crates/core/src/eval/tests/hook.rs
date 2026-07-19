//! Unit tests for the opt-in per-node evaluation hook (issue #732; span +
//! LAMBDA-parameter-binding enhancement per distributions ADR D10).
//!
//! These prove:
//! 1. it fires exactly once per evaluated node, in post-order, carrying the
//!    correct operation, span, and resulting value;
//! 2. lazy short-circuiting means only *evaluated* nodes fire (an unvisited
//!    `IF` branch never reports);
//! 3. wiring a hook (`Some`) does not change the computed value versus `None`;
//! 4. each `Span` matches the node's actual byte range in the source formula,
//!    and a child's span is always contained within its parent's;
//! 5. an `Apply`'s `LAMBDA` parameter fires an `EvalOp::Variable` binding
//!    event — even when the body never reads it — while the `LAMBDA` callee
//!    itself never fires as a discrete node.

use crate::eval::{Context, EvalCtx, EvalOp, Registry, Span};
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
/// and the ordered list of `(operation, span, resulting value)` events.
fn trace(formula: &str) -> (Value, Vec<(Op, Span, Value)>) {
    let engine = Engine::sheets();
    let expr = engine.parse(formula).expect("valid formula");
    let registry = Registry::new();

    let mut events: Vec<(Op, Span, Value)> = Vec::new();
    let value = {
        let mut record =
            |op: EvalOp<'_>, span: Span, v: &Value| events.push((Op::from(op), span, v.clone()));
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

/// Drop the span column — most tests only care about operation + value.
fn without_span(events: &[(Op, Span, Value)]) -> Vec<(Op, Value)> {
    events.iter().map(|(op, _, v)| (op.clone(), v.clone())).collect()
}

/// Slice `formula` with `span`: the exact source text a node's span covers.
/// This is exactly what a UI consumer does to highlight the formula
/// substring for a node (see `EvalHook` docs).
fn text_of<'a>(formula: &'a str, span: Span) -> &'a str {
    &formula[span.offset..span.offset + span.length]
}

/// `true` iff `child`'s byte range falls entirely inside `parent`'s — the
/// containment relationship a consumer uses to reconstruct a tree from the
/// flat post-order stream (see `EvalHook` docs).
fn contains(parent: Span, child: Span) -> bool {
    child.offset >= parent.offset
        && child.offset + child.length <= parent.offset + parent.length
}

#[test]
fn fires_post_order_for_binary_op() {
    // =1+2 → leaves fire before the operator; each event carries its value.
    let formula = "=1+2";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Number(3.0));
    assert_eq!(
        without_span(&events),
        vec![
            (Op::Number, Value::Number(1.0)),
            (Op::Number, Value::Number(2.0)),
            (Op::BinaryOp, Value::Number(3.0)),
        ]
    );
    // Each leaf's span covers just its own literal; the operator's span
    // covers the whole binary expression, containing both leaves.
    assert_eq!(text_of(formula, events[0].1), "1");
    assert_eq!(text_of(formula, events[1].1), "2");
    assert_eq!(text_of(formula, events[2].1), "1+2");
    assert!(contains(events[2].1, events[0].1));
    assert!(contains(events[2].1, events[1].1));
}

#[test]
fn fires_once_per_node_for_function_call() {
    // =SUM(1,2) → both arguments fire (post-order), then the call node.
    let formula = "=SUM(1,2)";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Number(3.0));
    assert_eq!(
        without_span(&events),
        vec![
            (Op::Number, Value::Number(1.0)),
            (Op::Number, Value::Number(2.0)),
            (Op::FunctionCall("SUM".to_string()), Value::Number(3.0)),
        ]
    );
    assert_eq!(text_of(formula, events[0].1), "1");
    assert_eq!(text_of(formula, events[1].1), "2");
    assert_eq!(text_of(formula, events[2].1), "SUM(1,2)");
    assert!(contains(events[2].1, events[0].1));
    assert!(contains(events[2].1, events[1].1));
}

#[test]
fn nested_tree_fires_every_node_exactly_once() {
    // =(1+2)*3 → 5 nodes: 1, 2, (1+2), 3, ((1+2)*3).
    let formula = "=(1+2)*3";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Number(9.0));
    assert_eq!(
        without_span(&events),
        vec![
            (Op::Number, Value::Number(1.0)),
            (Op::Number, Value::Number(2.0)),
            (Op::BinaryOp, Value::Number(3.0)),
            (Op::Number, Value::Number(3.0)),
            (Op::BinaryOp, Value::Number(9.0)),
        ]
    );
    assert_eq!(text_of(formula, events[0].1), "1");
    assert_eq!(text_of(formula, events[1].1), "2");
    assert_eq!(text_of(formula, events[2].1), "1+2");
    assert_eq!(text_of(formula, events[3].1), "3");
    assert_eq!(text_of(formula, events[4].1), "(1+2)*3");
    // Full containment: every earlier node's span is inside the final
    // (whole-expression) node's span, and the "1+2" node contains its own
    // two leaves.
    let root = events[4].1;
    for &(_, span, _) in &events {
        assert!(contains(root, span));
    }
    assert!(contains(events[2].1, events[0].1));
    assert!(contains(events[2].1, events[1].1));
}

#[test]
fn lazy_branch_not_taken_never_fires() {
    // =IF(TRUE, 10, 20) → only the condition, the taken branch, and the call
    // fire; the untaken `20` literal is never evaluated, so never reported.
    let formula = "=IF(TRUE, 10, 20)";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Number(10.0));
    assert_eq!(
        without_span(&events),
        vec![
            (Op::Bool, Value::Bool(true)),
            (Op::Number, Value::Number(10.0)),
            (Op::FunctionCall("IF".to_string()), Value::Number(10.0)),
        ]
    );
    // The value 20.0 must appear nowhere in the event stream.
    assert!(!events.iter().any(|(_, _, v)| *v == Value::Number(20.0)));
    // Fired spans point at the condition / taken-branch tokens, not at the
    // comma/whitespace/untaken-branch text around them.
    assert_eq!(text_of(formula, events[0].1), "TRUE");
    assert_eq!(text_of(formula, events[1].1), "10");
    assert!(contains(events[2].1, events[0].1));
    assert!(contains(events[2].1, events[1].1));
}

#[test]
fn unary_op_reports_operator_node() {
    let formula = "=-5";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Number(-5.0));
    assert_eq!(
        without_span(&events),
        vec![
            (Op::Number, Value::Number(5.0)),
            (Op::UnaryOp, Value::Number(-5.0)),
        ]
    );
    assert_eq!(text_of(formula, events[0].1), "5");
    assert_eq!(text_of(formula, events[1].1), "-5");
}

#[test]
fn text_and_concat_report_correct_ops() {
    let formula = "=\"a\"&\"b\"";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Text("ab".to_string()));
    assert_eq!(
        without_span(&events),
        vec![
            (Op::Text, Value::Text("a".to_string())),
            (Op::Text, Value::Text("b".to_string())),
            (Op::BinaryOp, Value::Text("ab".to_string())),
        ]
    );
    assert_eq!(text_of(formula, events[0].1), "\"a\"");
    assert_eq!(text_of(formula, events[1].1), "\"b\"");
    assert_eq!(text_of(formula, events[2].1), "\"a\"&\"b\"");
}

#[test]
fn array_literal_reports_elements_then_array() {
    let formula = "={1,2}";
    let (value, events) = trace(formula);
    assert_eq!(
        value,
        Value::Array(vec![Value::Number(1.0), Value::Number(2.0)])
    );
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].0, Op::Number);
    assert_eq!(events[1].0, Op::Number);
    assert_eq!(events[2].0, Op::Array);
    assert_eq!(text_of(formula, events[0].1), "1");
    assert_eq!(text_of(formula, events[1].1), "2");
    assert_eq!(text_of(formula, events[2].1), "{1,2}");
}

#[test]
fn error_node_still_fires_with_error_value() {
    // Argument-error short-circuit inside a function still reports the call
    // node — the hook observes the resulting error value.
    let (value, events) = trace("=SQRT(-1)");
    assert!(value.is_error());
    // Last event is the function-call node carrying the error result.
    let (last_op, _last_span, last_val) = events.last().expect("at least one event");
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
        "=LAMBDA(x, x*2)(5)",
    ] {
        let (with_hook, _) = trace(formula);
        let without_hook = eval_no_hook(formula);
        assert_eq!(with_hook, without_hook, "value diverged for {formula}");
    }
}

// ── Apply / LAMBDA callee (D10 / review finding F2) ─────────────────────────

#[test]
fn apply_fires_lambda_parameter_binding_even_when_body_reads_it() {
    // =LAMBDA(x, x*2)(5) → the call arg fires, then the parameter binding
    // (x=5) fires as its own Variable event before the body runs, then the
    // body's own read of `x` fires again (an ordinary Variable event, same
    // as any other variable read), then the body's operator, then Apply.
    let formula = "=LAMBDA(x, x*2)(5)";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Number(10.0));
    assert_eq!(
        without_span(&events),
        vec![
            (Op::Number, Value::Number(5.0)),
            (Op::Variable("x".to_string()), Value::Number(5.0)),
            (Op::Variable("x".to_string()), Value::Number(5.0)),
            (Op::Number, Value::Number(2.0)),
            (Op::BinaryOp, Value::Number(10.0)),
            (Op::Apply, Value::Number(10.0)),
        ]
    );
    // The parameter-binding event's span is the parameter's own token
    // position inside `LAMBDA(x, ...)` — not the call-site argument's span.
    assert_eq!(text_of(formula, events[1].1), "x");
    assert_ne!(events[0].1.offset, events[1].1.offset, "call arg and param token are different source positions");
    // The LAMBDA callee itself never fires as a discrete FunctionCall node —
    // the documented limitation (no Value represents a lambda).
    assert!(!events.iter().any(|(op, _, _)| matches!(op, Op::FunctionCall(name) if name == "LAMBDA")));
    // Every event's span is contained within the outer Apply's span.
    let apply_span = events.last().unwrap().1;
    assert_eq!(text_of(formula, apply_span), "LAMBDA(x, x*2)(5)");
    for &(_, span, _) in &events {
        assert!(contains(apply_span, span));
    }
}

#[test]
fn apply_fires_lambda_parameter_binding_even_when_body_never_reads_it() {
    // =LAMBDA(x, 42)(5) → body never references `x`, so without an explicit
    // binding event a trace would show the call arg (5) and the result (42)
    // with no visible link between them. The parameter-binding event closes
    // that gap: x=5 still fires even though nothing in the body reads it.
    let formula = "=LAMBDA(x, 42)(5)";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Number(42.0));
    assert_eq!(
        without_span(&events),
        vec![
            (Op::Number, Value::Number(5.0)),
            (Op::Variable("x".to_string()), Value::Number(5.0)),
            (Op::Number, Value::Number(42.0)),
            (Op::Apply, Value::Number(42.0)),
        ]
    );
    assert_eq!(text_of(formula, events[1].1), "x");
}

#[test]
fn apply_fires_a_binding_event_per_parameter_in_order() {
    // =LAMBDA(a, b, a-b)(10, 3) → two parameter-binding events, one per
    // parameter, each carrying its own argument.
    let formula = "=LAMBDA(a, b, a-b)(10, 3)";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Number(7.0));
    let bindings: Vec<(String, Value)> = events
        .iter()
        .filter_map(|(op, _, v)| match op {
            Op::Variable(name) => Some((name.clone(), v.clone())),
            _ => None,
        })
        .collect();
    // Two bindings (a=10, b=3) precede the body's own reads of a and b, which
    // also surface as Variable events with the same values — so at least the
    // first two Variable events are exactly the parameter bindings, in
    // declaration order.
    assert_eq!(bindings[0], ("a".to_string(), Value::Number(10.0)));
    assert_eq!(bindings[1], ("b".to_string(), Value::Number(3.0)));
}
