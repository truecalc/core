//! `INDEX` into a large inline array must cost the same whatever the array's
//! size (issue #894).
//!
//! `index_fn` used to *materialise a copy of the whole array* before reading
//! one element out of it: `flatten_to_rows` cloned every row, and the
//! one-dimensional branch then also ran `flatten_to_flat`, which allocates a
//! one-element `Vec` per element visited. One `INDEX(...)` into an
//! N-element inline array therefore cost O(N) heap allocations to return a
//! single scalar, so the natural workload — one lookup per row of the data —
//! cost O(N^2).
//!
//! The primary metric here is an **exact allocation count per `INDEX` call**,
//! not wall-clock time, so it is machine-independent, holds in either build
//! profile, and cannot go flaky. `index_fn` is called directly rather than
//! through `Engine::evaluate` so that parsing an N-element array literal —
//! which is legitimately O(N) — does not mask the number being asserted.
//!
//! Every measurement lives in one `#[test]` because a process-wide allocation
//! counter cannot be read from two tests running concurrently in the same
//! binary.

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

use truecalc_core::eval::functions::lookup::index_match::index_fn;
use truecalc_core::Value;

/// Every allocation the process makes. `realloc` is not overridden, so the
/// default `GlobalAlloc::realloc` routes through `alloc` and is counted too.
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Runs `body` and returns how many allocations it made.
fn allocations_during<T>(body: impl FnOnce() -> T) -> usize {
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    black_box(body());
    ALLOCATIONS.load(Ordering::Relaxed) - before
}

/// `{1,2,...,n}` — a one-row inline array of `n` numbers.
fn row_vector(n: usize) -> Value {
    Value::Array((1..=n).map(|i| Value::Number(i as f64)).collect())
}

/// `{1,2,3;4,5,6;...}` — `rows` rows of three numbers each.
fn grid(rows: usize) -> Value {
    Value::Array(
        (1..=rows)
            .map(|r| {
                Value::Array(
                    (1..=3)
                        .map(|c| Value::Number((r * 10 + c) as f64))
                        .collect(),
                )
            })
            .collect(),
    )
}

/// `{1;2;...;n}` — a column vector, i.e. `n` single-element rows.
fn column_vector(n: usize) -> Value {
    Value::Array(
        (1..=n)
            .map(|i| Value::Array(vec![Value::Number(i as f64)]))
            .collect(),
    )
}

const SIZES: [usize; 5] = [100, 500, 1_000, 5_000, 10_000];

/// Allocations made by one `INDEX(array, row, [col])` that returns one scalar.
fn allocations_for_one_lookup(array: Value, row: usize, col: Option<usize>) -> usize {
    let mut args = vec![array, Value::Number(row as f64)];
    if let Some(c) = col {
        args.push(Value::Number(c as f64));
    }
    // Warm any lazily-initialised state so it is not billed to the measurement.
    black_box(index_fn(&args));
    allocations_during(|| index_fn(&args))
}

#[test]
fn index_into_an_inline_array_allocates_a_constant_amount() {
    // ── Row vector: INDEX({1..n}, k) ────────────────────────────────────────
    let row_counts: Vec<(usize, usize)> = SIZES
        .iter()
        .map(|&n| (n, allocations_for_one_lookup(row_vector(n), n / 2, None)))
        .collect();

    // ── Grid: INDEX({..n rows x 3..}, k, 2) ─────────────────────────────────
    let grid_counts: Vec<(usize, usize)> = SIZES
        .iter()
        .map(|&n| (n, allocations_for_one_lookup(grid(n), n / 2, Some(2))))
        .collect();

    // ── Column vector: INDEX({1;..;n}, k, 1) ────────────────────────────────
    let column_counts: Vec<(usize, usize)> = SIZES
        .iter()
        .map(|&n| {
            (
                n,
                allocations_for_one_lookup(column_vector(n), n / 2, Some(1)),
            )
        })
        .collect();

    let report = |label: &str, counts: &[(usize, usize)]| {
        let cells: Vec<String> = counts.iter().map(|(n, a)| format!("{n}: {a}")).collect();
        format!(
            "{label} — allocations per INDEX call by array size: {}",
            cells.join(", ")
        )
    };

    let measured = [
        ("row vector", &row_counts),
        ("grid", &grid_counts),
        ("column vector", &column_counts),
    ];
    for (label, counts) in measured {
        println!("{}", report(label, counts));
    }

    for (label, counts) in measured {
        let first = counts[0].1;
        for &(n, allocs) in counts {
            assert_eq!(
                allocs,
                first,
                "{label}: INDEX allocated {allocs} times at size {n} but {first} at size {}. \
                 The cost of reading one element must not depend on how big the array is. \
                 {}",
                counts[0].0,
                report(label, counts),
            );
        }

        // A single-element read needs no heap at all. Pinning the absolute
        // number (not just its flatness) is what catches a regression that
        // reintroduces a fixed-size copy of the array — flat, but O(N) work.
        assert_eq!(
            first,
            0,
            "{label}: reading one element out of an array must not allocate. {}",
            report(label, counts),
        );
    }

    // ── The shape the issue is actually about ───────────────────────────────
    // A caller passes a data set in as one inline array and reads every row of
    // it. With an O(N) cost per lookup that whole pass is O(N^2); the count
    // below is what makes the difference between "linear with a bad constant"
    // and "quadratic" visible without a stopwatch.
    let pass_counts: Vec<(usize, usize)> = SIZES
        .iter()
        .map(|&n| {
            let array = grid(n);
            let mut args = vec![array, Value::Number(1.0), Value::Number(2.0)];
            black_box(index_fn(&args));
            let allocs = allocations_during(|| {
                for row in 1..=n {
                    args[1] = Value::Number(row as f64);
                    black_box(index_fn(&args));
                }
            });
            (n, allocs)
        })
        .collect();

    let pass_report: Vec<String> = pass_counts
        .iter()
        .map(|(n, a)| format!("{n} rows: {a}"))
        .collect();
    println!(
        "one lookup per row — total allocations for the whole pass: {}",
        pass_report.join(", ")
    );

    for &(n, allocs) in &pass_counts {
        assert_eq!(
            allocs,
            0,
            "reading all {n} rows of an inline array allocated {allocs} times; \
             it must not allocate at all. {}",
            pass_report.join(", "),
        );
    }
}
