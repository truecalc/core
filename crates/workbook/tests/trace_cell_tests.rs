//! `Workbook::trace_cell` (issue #743): explains a single cell's value against
//! the current stored grid — reusing the exact grid-backed resolver semantics
//! `recalc` uses for that cell's precedents — with an opt-in per-node
//! `EvalHook` wired through to core's `evaluate_expr` (issue #732/#743's
//! seam) so a caller can observe the evaluation, not just its final value.

use truecalc_core::eval::EvalOp;
use truecalc_core::{Span, Value as CoreValue};
use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

/// A fixed, DST-free context (GMT) pinned to an arbitrary instant — same
/// convention as `recalc_tests.rs`.
fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn a1(s: &str) -> Address {
    Address::from_a1(s).expect("valid A1")
}

fn sheets_wb() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb
}

#[test]
fn trace_cell_matches_recalc_and_observes_a_post_order_event_stream() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1*2".into()))
        .unwrap();

    wb.recalc(&ctx());
    let recalced = wb.get("Sheet1", a1("B1")).unwrap().value().clone();
    assert_eq!(
        recalced,
        Value::Number(10.0),
        "sanity: recalc computed B1 = 10"
    );

    // Record every event as (is-the-A1-read, resulting core value), in
    // firing order — flattened immediately since `EvalOp` borrows from the
    // AST and cannot outlive the hook call. A *bare* cell reference like
    // `A1` parses as `Expr::Variable("A1", _)` (only sheet-qualified refs
    // such as `Sheet1!A1` parse as `Expr::Reference`, per `parser::ast::Expr`
    // docs), so the read fires as `EvalOp::Variable("A1")`, resolved through
    // the same unbound-reference path a real `Expr::Reference` would use.
    let mut events: Vec<(bool, CoreValue)> = Vec::new();
    let mut hook = |op: EvalOp<'_>, _span: Span, value: &CoreValue| {
        events.push((matches!(op, EvalOp::Variable("A1")), value.clone()));
    };

    let traced = wb.trace_cell("Sheet1", a1("B1"), &ctx(), &mut hook);

    // (a) the returned value matches what recalc computed for B1.
    assert_eq!(traced, Value::Number(10.0));
    assert_eq!(traced, recalced);

    // (b) a well-formed post-order stream for `=A1*2`: the A1 reference leaf,
    // then the `2` literal, then the top-level multiplication — children
    // strictly before the parent that consumes them, and the last event's
    // value matches the overall result.
    assert_eq!(
        events,
        vec![
            (true, CoreValue::Number(5.0)),
            (false, CoreValue::Number(2.0)),
            (false, CoreValue::Number(10.0)),
        ]
    );

    // (c) the reference leaf for A1 carries A1's resolved value (5).
    let a1_events: Vec<&CoreValue> = events
        .iter()
        .filter(|(is_a1, _)| *is_a1)
        .map(|(_, v)| v)
        .collect();
    assert_eq!(a1_events, vec![&CoreValue::Number(5.0)]);
}

#[test]
fn trace_cell_with_a_no_op_hook_still_matches_recalc() {
    // "No hook attached ⇒ identical value": `trace_cell` always takes a
    // hook (there is no `Option`-shaped overload at the workbook layer), so
    // the no-observation case is exercised with a hook that never inspects
    // its arguments — proving `trace_cell`'s *value* is independent of
    // whether the caller does anything with the events. The stronger,
    // `hook: None`-vs-`Some` equivalence at the seam itself is asserted at
    // the core layer (`crates/core/src/engine/tests.rs`,
    // `hooked_keyed_resolver_eval_with_none_hook_matches_unhooked_method`).
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=A1*2".into()))
        .unwrap();
    wb.recalc(&ctx());

    let mut no_op = |_op: EvalOp<'_>, _span: Span, _value: &CoreValue| {};
    let traced = wb.trace_cell("Sheet1", a1("B1"), &ctx(), &mut no_op);
    assert_eq!(traced, Value::Number(10.0));
}

#[test]
fn trace_cell_on_a_literal_cell_returns_its_stored_value_without_firing_the_hook() {
    // A literal has no expression to trace: `trace_cell` returns its stored
    // value directly and never invokes the hook (documented decision).
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    wb.recalc(&ctx());

    let mut fired = false;
    let mut hook = |_op: EvalOp<'_>, _span: Span, _value: &CoreValue| {
        fired = true;
    };
    let traced = wb.trace_cell("Sheet1", a1("A1"), &ctx(), &mut hook);

    assert_eq!(traced, Value::Number(5.0));
    assert!(
        !fired,
        "a literal cell has no expression, so the hook must not fire"
    );
}

#[test]
fn trace_cell_on_an_empty_cell_returns_empty_without_firing_the_hook() {
    let wb = sheets_wb();

    let mut fired = false;
    let mut hook = |_op: EvalOp<'_>, _span: Span, _value: &CoreValue| {
        fired = true;
    };
    let traced = wb.trace_cell("Sheet1", a1("Z9"), &ctx(), &mut hook);

    assert_eq!(traced, Value::Empty);
    assert!(!fired);
}
