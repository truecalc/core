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
//!    itself never fires as a discrete node;
//! 6. the same parameter-binding event fires for the six higher-order array
//!    functions (MAP/REDUCE/BYROW/BYCOL/SCAN/MAKEARRAY), which bind lambda
//!    parameters through their own `apply_lambda` helper rather than
//!    `eval_apply` — once per parameter, per invocation.

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

// ── Higher-order functions (MAP/REDUCE/BYROW/BYCOL/SCAN/MAKEARRAY) bind
// lambda parameters through their own `apply_lambda` helper, not
// `eval_apply` — this is the gap #740's review found (issue follow-up).
// These prove each HOF now fires one parameter-binding event per parameter
// per invocation, even when the body never reads the parameter.

/// Collect just the `Variable` events (name, value) from a trace, in order —
/// the parameter-binding events plus any ordinary in-body variable reads.
fn variable_events(events: &[(Op, Span, Value)]) -> Vec<(String, Value)> {
    events
        .iter()
        .filter_map(|(op, _, v)| match op {
            Op::Variable(name) => Some((name.clone(), v.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn map_fires_one_parameter_event_per_element_even_when_body_ignores_it() {
    // =MAP({1,2,3}, LAMBDA(x, 42)) → body never reads x, but the gap this PR
    // closes means each of the 3 invocations must still fire its own x=<n>
    // binding event (values 1, 2, 3 — one per element).
    let formula = "=MAP({1,2,3}, LAMBDA(x, 42))";
    let (value, events) = trace(formula);
    assert_eq!(
        value,
        Value::Array(vec![Value::Number(42.0), Value::Number(42.0), Value::Number(42.0)])
    );
    let bindings = variable_events(&events);
    assert_eq!(
        bindings,
        vec![
            ("x".to_string(), Value::Number(1.0)),
            ("x".to_string(), Value::Number(2.0)),
            ("x".to_string(), Value::Number(3.0)),
        ]
    );
    // All three share the parameter's own span (its token position inside
    // `LAMBDA(x, ...)`), not deduped — one event per invocation by design.
    let param_spans: Vec<Span> = events
        .iter()
        .filter(|(op, _, _)| matches!(op, Op::Variable(n) if n == "x"))
        .map(|(_, span, _)| *span)
        .collect();
    assert_eq!(param_spans.len(), 3);
    assert_eq!(param_spans[0], param_spans[1]);
    assert_eq!(param_spans[1], param_spans[2]);
    assert_eq!(text_of(formula, param_spans[0]), "x");
}

#[test]
fn map_fires_parameter_events_when_body_reads_it_too() {
    // =MAP({1,2,3}, LAMBDA(x, x*10)) → each invocation fires the binding
    // event (x=1/2/3) followed by the body's own read of x (same value).
    let formula = "=MAP({1,2,3}, LAMBDA(x, x*10))";
    let (value, events) = trace(formula);
    assert_eq!(
        value,
        Value::Array(vec![Value::Number(10.0), Value::Number(20.0), Value::Number(30.0)])
    );
    let bindings = variable_events(&events);
    // Binding event + body-read event per invocation, each pair equal.
    assert_eq!(bindings.len(), 6);
    for pair in bindings.chunks(2) {
        assert_eq!(pair[0], pair[1]);
    }
    assert_eq!(bindings[0].1, Value::Number(1.0));
    assert_eq!(bindings[2].1, Value::Number(2.0));
    assert_eq!(bindings[4].1, Value::Number(3.0));
}

#[test]
fn reduce_fires_two_parameter_events_per_item() {
    // =REDUCE(0, {1,2,3}, LAMBDA(acc,item, acc+item)) → 3 invocations, each
    // binding both acc and item. The body reads both, so each parameter
    // fires twice per invocation (the bind-time event, then the body's own
    // read) — take every other (even-indexed) event to isolate the bindings.
    let formula = "=REDUCE(0, {1,2,3}, LAMBDA(acc,item, acc+item))";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Number(6.0));
    let bindings = variable_events(&events);
    let acc_bindings: Vec<Value> = bindings
        .iter()
        .filter(|(n, _)| n == "acc")
        .step_by(2)
        .map(|(_, v)| v.clone())
        .collect();
    let item_bindings: Vec<Value> = bindings
        .iter()
        .filter(|(n, _)| n == "item")
        .step_by(2)
        .map(|(_, v)| v.clone())
        .collect();
    // acc: 0, 1, 3 (running total before each step); item: 1, 2, 3.
    assert_eq!(
        acc_bindings,
        vec![Value::Number(0.0), Value::Number(1.0), Value::Number(3.0)]
    );
    assert_eq!(
        item_bindings,
        vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
    );
}

#[test]
fn scan_fires_two_parameter_events_per_item() {
    // =SCAN(0, {1,2,3}, LAMBDA(acc,item, acc+item)) → same binding shape as
    // REDUCE, but SCAN also returns the running values. Body reads both
    // params, so each fires twice per invocation; every other event is the
    // bind-time one.
    let formula = "=SCAN(0, {1,2,3}, LAMBDA(acc,item, acc+item))";
    let (value, events) = trace(formula);
    assert_eq!(
        value,
        Value::Array(vec![Value::Number(1.0), Value::Number(3.0), Value::Number(6.0)])
    );
    let bindings = variable_events(&events);
    let item_bindings: Vec<Value> = bindings
        .iter()
        .filter(|(n, _)| n == "item")
        .step_by(2)
        .map(|(_, v)| v.clone())
        .collect();
    assert_eq!(
        item_bindings,
        vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
    );
}

#[test]
fn byrow_fires_one_parameter_event_per_row_even_when_body_ignores_it() {
    // =BYROW({1,2;3,4}, LAMBDA(row, 99)) → 2 rows, body ignores `row`, but
    // each invocation still fires row=<array> even though nothing reads it.
    let formula = "=BYROW({1,2;3,4}, LAMBDA(row, 99))";
    let (value, events) = trace(formula);
    assert_eq!(
        value,
        Value::Array(vec![
            Value::Array(vec![Value::Number(99.0)]),
            Value::Array(vec![Value::Number(99.0)]),
        ])
    );
    let bindings = variable_events(&events);
    assert_eq!(
        bindings,
        vec![
            (
                "row".to_string(),
                Value::Array(vec![Value::Number(1.0), Value::Number(2.0)])
            ),
            (
                "row".to_string(),
                Value::Array(vec![Value::Number(3.0), Value::Number(4.0)])
            ),
        ]
    );
}

#[test]
fn bycol_fires_one_parameter_event_per_column_even_when_body_ignores_it() {
    // =BYCOL({1,2;3,4}, LAMBDA(col, 99)) → 2 columns.
    let formula = "=BYCOL({1,2;3,4}, LAMBDA(col, 99))";
    let (value, events) = trace(formula);
    assert_eq!(value, Value::Array(vec![Value::Number(99.0), Value::Number(99.0)]));
    let bindings = variable_events(&events);
    assert_eq!(
        bindings,
        vec![
            (
                "col".to_string(),
                Value::Array(vec![Value::Number(1.0), Value::Number(3.0)])
            ),
            (
                "col".to_string(),
                Value::Array(vec![Value::Number(2.0), Value::Number(4.0)])
            ),
        ]
    );
}

#[test]
fn makearray_fires_two_parameter_events_per_cell_even_when_body_ignores_them() {
    // =MAKEARRAY(2, 2, LAMBDA(r,c, 0)) → 4 cells, body ignores both params,
    // but each invocation still fires r=<row> and c=<col>.
    let formula = "=MAKEARRAY(2, 2, LAMBDA(r,c, 0))";
    let (value, events) = trace(formula);
    assert_eq!(
        value,
        Value::Array(vec![
            Value::Array(vec![Value::Number(0.0), Value::Number(0.0)]),
            Value::Array(vec![Value::Number(0.0), Value::Number(0.0)]),
        ])
    );
    let bindings = variable_events(&events);
    let r_events: Vec<Value> =
        bindings.iter().filter(|(n, _)| n == "r").map(|(_, v)| v.clone()).collect();
    let c_events: Vec<Value> =
        bindings.iter().filter(|(n, _)| n == "c").map(|(_, v)| v.clone()).collect();
    assert_eq!(
        r_events,
        vec![
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(2.0)
        ]
    );
    assert_eq!(
        c_events,
        vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(1.0),
            Value::Number(2.0)
        ]
    );
}

#[test]
fn hof_parameter_events_do_not_change_computed_value() {
    // The hook is purely observational for the HOF path too: computed values
    // match with and without a hook wired.
    for formula in [
        "=MAP({1,2,3}, LAMBDA(x, x*2))",
        "=REDUCE(0, {1,2,3}, LAMBDA(acc,item, acc+item))",
        "=SCAN(0, {1,2,3}, LAMBDA(acc,item, acc+item))",
        "=BYROW({1,2;3,4}, LAMBDA(row, SUM(row)))",
        "=BYCOL({1,2;3,4}, LAMBDA(col, SUM(col)))",
        "=MAKEARRAY(2, 2, LAMBDA(r,c, r*10+c))",
    ] {
        let (with_hook, _) = trace(formula);
        let without_hook = eval_no_hook(formula);
        assert_eq!(with_hook, without_hook, "value diverged for {formula}");
    }
}
