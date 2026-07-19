//! Per-node evaluation-hook overhead benchmark (issue #732).
//!
//! The acceptance criterion asks for the enabled-path per-node cost, which
//! feeds the compute-cost model. We evaluate one fixed expression three ways:
//!
//! * `none`     — `EvalCtx::hook = None` (the free/unmetered path);
//! * `counter`  — a hook that only increments a node counter (minimal work);
//! * `collect`  — a hook that clones each `(op-tag, Value)` into a `Vec`
//!                (a realistic tracing/profiling consumer's cost).
//!
//! Divide the `none → counter` delta by the node count (printed once at start)
//! to get the raw per-node hook overhead; `collect` shows a realistic upper
//! bound including the consumer's own per-node work.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use truecalc_core::eval::{evaluate_expr, Context, EvalCtx, EvalOp, Registry};
use truecalc_core::types::Value;
use truecalc_core::Engine;

/// A wide + deep arithmetic tree: many leaves and operators, no I/O, so the
/// benchmark isolates tree-walk + hook cost rather than function internals.
const FORMULA: &str = "=((1+2)*(3-4)+(5*6)/(7+8))*((9-1)+(2*3))-(4+5)*(6-7)+(8*9)/(1+1)";

/// Count how many nodes the hook observes for `FORMULA` (one event per node).
fn node_count(expr: &truecalc_core::Expr, registry: &Registry) -> usize {
    let mut n = 0usize;
    let mut count = |_op: EvalOp<'_>, _v: &Value| n += 1;
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

    let mut group = c.benchmark_group("eval_hook");

    // Free/unmetered path: no hook wired.
    group.bench_function("none", |b| {
        b.iter(|| {
            let mut ctx = EvalCtx::new(Context::empty(), &registry);
            black_box(evaluate_expr(black_box(&expr), &mut ctx))
        });
    });

    // Enabled path, minimal consumer: count nodes only.
    group.bench_function("counter", |b| {
        b.iter(|| {
            let mut n = 0usize;
            let mut count = |_op: EvalOp<'_>, _v: &Value| n += 1;
            let mut ctx = EvalCtx::new(Context::empty(), &registry);
            ctx.hook = Some(&mut count);
            let out = evaluate_expr(black_box(&expr), &mut ctx);
            black_box(n);
            black_box(out)
        });
    });

    // Enabled path, realistic consumer: clone each (op-tag, value) into a Vec.
    group.bench_function("collect", |b| {
        b.iter(|| {
            let mut events: Vec<(u8, Value)> = Vec::new();
            let mut collect = |op: EvalOp<'_>, v: &Value| {
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
                };
                events.push((tag, v.clone()));
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
