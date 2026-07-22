//! Per-node evaluation-hook overhead benchmark (issue #732; span-carrying
//! enhancement per distributions ADR D10).
//!
//! The acceptance criterion asks for the enabled-path per-node cost, which
//! feeds the compute-cost model. We evaluate one fixed expression four ways:
//!
//! * `none`         — `EvalCtx::hook = None` (the free/unmetered path);
//! * `counter`       — a hook that only increments a node counter, ignoring
//!                     the span entirely (minimal work, isolates the base
//!                     per-node overhead: branch + `EvalOp::of` + vtable call);
//! * `counter_span`  — the same counter, but also folds `span.offset +
//!                     span.length` into an accumulator (isolates the
//!                     incremental cost of the span parameter itself — `Span`
//!                     is `Copy`, two `usize`s, so this should be ~free);
//! * `collect`       — a hook that clones each `(op-tag, span, Value)` into a
//!                     `Vec` (a realistic tracing/profiling consumer's cost,
//!                     the shape a span-reconstructing consumer like
//!                     core-pro's trace tree actually pays).
//!
//! Divide the `none → counter` delta by the node count to get the raw
//! per-node hook overhead pre-span; `counter → counter_span` isolates the
//! span-carrying delta specifically; `collect` shows a realistic upper bound
//! including the consumer's own per-node work.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use truecalc_core::eval::{evaluate_expr, Context, EvalCtx, EvalOp, Registry, Span};
use truecalc_core::types::Value;
use truecalc_core::Engine;

/// A wide + deep arithmetic tree: many leaves and operators, no I/O, so the
/// benchmark isolates tree-walk + hook cost rather than function internals.
const FORMULA: &str = "=((1+2)*(3-4)+(5*6)/(7+8))*((9-1)+(2*3))-(4+5)*(6-7)+(8*9)/(1+1)";

/// Count how many nodes the hook observes for `FORMULA` (one event per node).
fn node_count(expr: &truecalc_core::Expr, registry: &Registry) -> usize {
    let mut n = 0usize;
    let mut count = |_op: EvalOp<'_>, _span: Span, _v: &Value| n += 1;
    let mut ctx = EvalCtx::new(Context::empty(), registry);
    ctx.hook = Some(&mut count);
    let _ = evaluate_expr(expr, &mut ctx);
    n
}

fn bench_hook(c: &mut Criterion) {
    let engine = Engine::sheets();
    let expr = engine.parse(FORMULA).expect("valid formula");
    let registry = Registry::new();

    let nodes = node_count(&expr, &registry);
    println!("eval_hook bench: {nodes} nodes evaluated per iteration");
    println!("size_of::<Span>() = {} bytes", std::mem::size_of::<Span>());

    let mut group = c.benchmark_group("eval_hook");

    // Free/unmetered path: no hook wired.
    group.bench_function("none", |b| {
        b.iter(|| {
            let mut ctx = EvalCtx::new(Context::empty(), &registry);
            black_box(evaluate_expr(black_box(&expr), &mut ctx))
        });
    });

    // Enabled path, minimal consumer: count nodes only, never touch the span.
    group.bench_function("counter", |b| {
        b.iter(|| {
            let mut n = 0usize;
            let mut count = |_op: EvalOp<'_>, _span: Span, _v: &Value| n += 1;
            let mut ctx = EvalCtx::new(Context::empty(), &registry);
            ctx.hook = Some(&mut count);
            let out = evaluate_expr(black_box(&expr), &mut ctx);
            black_box(n);
            black_box(out)
        });
    });

    // Enabled path, minimal consumer that also touches the span (isolates the
    // span-carrying delta specifically — `Span` is `Copy`, so this should add
    // only a couple of register-sized additions per node).
    group.bench_function("counter_span", |b| {
        b.iter(|| {
            let mut n = 0usize;
            let mut span_acc = 0usize;
            let mut count = |_op: EvalOp<'_>, span: Span, _v: &Value| {
                n += 1;
                span_acc = span_acc.wrapping_add(span.offset).wrapping_add(span.length);
            };
            let mut ctx = EvalCtx::new(Context::empty(), &registry);
            ctx.hook = Some(&mut count);
            let out = evaluate_expr(black_box(&expr), &mut ctx);
            black_box(n);
            black_box(span_acc);
            black_box(out)
        });
    });

    // Enabled path, realistic consumer: clone each (op-tag, span, value) into
    // a Vec — the shape a span-reconstructing trace-tree consumer pays.
    group.bench_function("collect", |b| {
        b.iter(|| {
            let mut events: Vec<(u8, Span, Value)> = Vec::new();
            let mut collect = |op: EvalOp<'_>, span: Span, v: &Value| {
                let tag = match op {
                    EvalOp::Number => 0,
                    EvalOp::Text => 1,
                    EvalOp::Bool => 2,
                    EvalOp::Variable(_) => 3,
                    EvalOp::Reference => 4,
                    EvalOp::UnaryOp(_) => 5,
                    EvalOp::BinaryOp(_) => 6,
                    EvalOp::Array => 7,
                    EvalOp::Apply => 8,
                    EvalOp::FunctionCall(_) => 9,
                    EvalOp::Error(_) => 10,
                };
                events.push((tag, span, v.clone()));
            };
            let mut ctx = EvalCtx::new(Context::empty(), &registry);
            ctx.hook = Some(&mut collect);
            let out = evaluate_expr(black_box(&expr), &mut ctx);
            black_box(&events);
            black_box(out)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_hook);
criterion_main!(benches);
