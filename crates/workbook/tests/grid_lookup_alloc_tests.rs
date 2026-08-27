//! Allocation-free grid lookups (issue #887).
//!
//! `Worksheet` keys its sparse grid by plain A1 `String`s. Every read-side
//! accessor used to build that key with `Address::to_a1()` — three heap
//! allocations (the column letters, then the row digits, then the `push_str`
//! regrow that joins them) on **every** cell access. A formula that scans a
//! range pays this once per element visited, so
//! a workbook of range-reading formulas paid it O(cells scanned) times per
//! pass. `BTreeMap<String, _>` probes through `Borrow<str>`, so the owned key
//! was never needed: the accessors now render the key into a stack buffer.
//!
//! These tests pin the change in **allocation counts**, not wall-clock time, so
//! they hold on any machine and in either build profile and cannot go flaky.
//! Both live in one `#[test]` because a process-wide allocation counter cannot
//! be read from two tests running concurrently in the same binary.

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

use truecalc_workbook::{
    Address, Cell, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

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

/// `Sheet1` with `A1..A{rows}` holding `1..rows` as literals.
fn column_of_literals(rows: u32) -> Worksheet {
    let mut sheet = Worksheet::new("Sheet1");
    for row in 1..=rows {
        sheet.cells_mut().insert(
            Address::new(row, 1).unwrap().to_a1(),
            Cell::literal(Value::Number(f64::from(row))).unwrap(),
        );
    }
    sheet
}

/// A workbook whose single formula sums the whole of `A1:A{rows}`, so a recalc
/// scans exactly `rows` elements.
fn scan_of(rows: u32) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(column_of_literals(rows)).unwrap();
    wb.set(
        "Sheet1",
        Address::new(1, 2).unwrap(),
        CellInput::Formula(format!("=SUM(A$1:A{rows})")),
    )
    .unwrap();
    wb
}

fn allocations_to_recalc(rows: u32) -> usize {
    let ctx = RecalcContext::new(0, "UTC", 0).unwrap();
    let mut wb = scan_of(rows);
    allocations_during(|| wb.recalc(&ctx))
}

/// Measured on this change at `LOOKUPS = 10_000`: 60_000 allocations before
/// (three per `to_a1`), 0 after.
const LOOKUPS: u32 = 10_000;

/// The marginal cost of scanning one more range element, in allocations.
/// Measured by differencing two recalcs so the fixed per-recalc cost cancels:
/// 12.00 before this change, 6.00 after, and 2.00 once #904 removed
/// `GridResolver::cell_value`'s own per-element allocations too.
///
/// It is not 0 because the range measured here is a single column, which is
/// materialized as core's Nx1 column shape — one nested one-element `Array` per
/// row, a `Vec` per element by construction. That is a deliberate semantic, not
/// an allocation defect; `cell_value_alloc_tests.rs` measures a block range,
/// where the same scan allocates nothing at all. This budget is what pins the
/// A1-key share of it from coming back.
const MAX_ALLOCATIONS_PER_ELEMENT_SCANNED: f64 = 3.0;

#[test]
fn grid_lookups_do_not_allocate_an_a1_key() {
    // ── 1. A grid read allocates nothing at all ──────────────────────────
    let sheet = column_of_literals(LOOKUPS);
    let addresses: Vec<Address> = (1..=LOOKUPS).map(|r| Address::new(r, 1).unwrap()).collect();

    // Warm up so first-touch costs are not attributed to the measured run.
    for addr in &addresses {
        black_box(sheet.get(*addr));
    }

    let hits = allocations_during(|| {
        for addr in &addresses {
            black_box(sheet.get(*addr));
            black_box(sheet.contains(*addr));
        }
    });
    assert_eq!(
        hits, 0,
        "{LOOKUPS} grid lookups allocated {hits} times; a lookup must key the \
         cell map by a borrowed &str, never by an owned A1 String"
    );

    // ── 2. A range scan stays within its per-element budget ──────────────
    // Warm up the recalc path (lazy statics, the function registry) first.
    black_box(allocations_to_recalc(64));

    let small = allocations_to_recalc(LOOKUPS / 10);
    let large = allocations_to_recalc(LOOKUPS / 5);
    let extra_elements = f64::from(LOOKUPS / 5 - LOOKUPS / 10);
    let per_element = (large - small) as f64 / extra_elements;

    assert!(
        per_element <= MAX_ALLOCATIONS_PER_ELEMENT_SCANNED,
        "scanning one more range element cost {per_element:.2} allocations \
         (budget {MAX_ALLOCATIONS_PER_ELEMENT_SCANNED:.2}); \
         {small} allocations for {} elements, {large} for {}",
        LOOKUPS / 10,
        LOOKUPS / 5
    );
}
