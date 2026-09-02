//! Cache correctness for the volatile-cell set `CachedGraph` now carries
//! (issue #983): the set must be exactly right after every graph rebuild, not
//! merely "usually right until an edit exercises the bug."
//!
//! ## Why behavioral, not structural
//!
//! `CachedGraph::volatile` is `pub(crate)`, matching `order`/`cycle` (neither
//! is exposed either), so this asserts through the same instrumentation the
//! rest of the incremental-recalc suite already uses:
//!
//!  * [`Workbook::graph_builds`] — an exact count of how many graphs have
//!    actually been built, proving a rebuild happened (and, just as
//!    important, that a later call reused the warm entry rather than
//!    rebuilding it every time);
//!  * `recalc_incremental_measured`'s returned closure size — the dirty set
//!    an edit produced, which is exactly what a stale volatile entry would
//!    get wrong.
//!
//! This is also the more convincing check: it catches the exact failure the
//! issue calls out — "a stale 'not volatile' entry producing a wrong recalc
//! result that looks fine but is stale" — rather than an internal flag that
//! could be right for the wrong reason.
//!
//! ## Fixture
//!
//! One sheet. `A1` a literal, `B1 = "=A1+1"` (a plain, never-volatile
//! formula), `C1` a lone literal used as the "unrelated edit" target in every
//! case, so the edit's own direct dependents are empty and the *only* thing
//! that can land in the dirty closure is whatever the volatile-seeding step
//! (or lack of it) contributes.

use truecalc_workbook::{
    Address, CellInput, Change, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).expect("valid A1")
}

/// `A1` = 1 (literal), `B1` = `=A1+1` (formula, never volatile), `C1` = 0
/// (literal, the unrelated edit target).
fn base_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb.set("Sheet1", addr("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("Sheet1", addr("B1"), CellInput::Formula("=A1+1".into()))
        .unwrap();
    wb.set("Sheet1", addr("C1"), CellInput::Literal(Value::Number(0.0)))
        .unwrap();
    wb
}

/// Writes a new literal into `C1` (an edit unrelated to anything under test —
/// it has no dependents) and runs an incremental recalc seeded from it,
/// returning the changes and the dirty-closure size.
fn edit_c1(wb: &mut Workbook, n: f64) -> (Vec<Change>, usize) {
    wb.set("Sheet1", addr("C1"), CellInput::Literal(Value::Number(n)))
        .unwrap();
    wb.recalc_incremental_measured(&ctx(), &[("Sheet1".to_string(), addr("C1"))])
}

fn touched(changes: &[Change]) -> Vec<String> {
    changes.iter().map(|c| c.addr.to_a1()).collect()
}

// ---------------------------------------------------------------------------
// (a) Adding a new volatile formula.
// ---------------------------------------------------------------------------

#[test]
fn adding_a_volatile_formula_puts_it_in_the_cached_set() {
    let mut wb = base_workbook();
    wb.recalc(&ctx());
    assert_eq!(wb.graph_builds(), 1);

    // D1 was previously empty; adding "=NOW()" both rebuilds the graph (a new
    // formula cell) and must recompute the volatile set for it.
    wb.set("Sheet1", addr("D1"), CellInput::Formula("=NOW()".into()))
        .unwrap();
    assert!(
        !wb.graph_cache_is_warm(),
        "a new formula cell invalidates the graph cache"
    );

    let (changes, closure) = edit_c1(&mut wb, 1.0);
    assert_eq!(wb.graph_builds(), 2, "the graph rebuilt exactly once");
    assert_eq!(
        closure, 1,
        "the dirty closure must be exactly the volatile cell D1 — the C1 \
         edit itself contributes nothing else"
    );
    assert!(
        touched(&changes).contains(&"D1".to_string()),
        "{:?}",
        touched(&changes)
    );

    // Repeating the same kind of edit with no further mutation must reuse
    // the warm cache (no rebuild) while still finding D1 volatile every time
    // — proving this is a cache, not a rebuild-every-call in disguise.
    let (_changes2, closure2) = edit_c1(&mut wb, 2.0);
    assert_eq!(
        wb.graph_builds(),
        2,
        "no mutation happened since the last build; the cache must stay warm"
    );
    assert_eq!(closure2, 1, "D1 must still be seeded as volatile");
}

// ---------------------------------------------------------------------------
// (b) Removing a volatile formula.
// ---------------------------------------------------------------------------

#[test]
fn removing_a_volatile_formula_drops_it_from_the_cached_set() {
    let mut wb = base_workbook();
    wb.set("Sheet1", addr("D1"), CellInput::Formula("=NOW()".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(wb.graph_builds(), 1);

    // Clear D1 back to empty: it is no longer a formula cell at all.
    wb.clear("Sheet1", addr("D1"));
    assert!(
        !wb.graph_cache_is_warm(),
        "clearing a formula cell invalidates the graph cache"
    );

    let (changes, closure) = edit_c1(&mut wb, 1.0);
    assert_eq!(wb.graph_builds(), 2);
    assert_eq!(
        closure, 0,
        "D1 is gone; the cached volatile set must not retain a stale entry \
         for it"
    );
    assert!(
        !touched(&changes).contains(&"D1".to_string()),
        "{:?}",
        touched(&changes)
    );
}

// ---------------------------------------------------------------------------
// (c) Flipping volatility on an existing formula cell, both directions.
// ---------------------------------------------------------------------------

#[test]
fn flipping_a_formula_between_volatile_and_not_updates_the_cached_set_both_ways() {
    let mut wb = base_workbook();
    wb.recalc(&ctx());
    assert_eq!(wb.graph_builds(), 1);

    // B1 = "=A1+1" is not volatile: an unrelated C1 edit has nothing to seed.
    let (_changes, closure) = edit_c1(&mut wb, 1.0);
    assert_eq!(
        wb.graph_builds(),
        1,
        "no formula changed; the graph stays warm"
    );
    assert_eq!(closure, 0, "B1 is not volatile yet");

    // Flip B1 to a volatile formula over the same cell (a formula write, so
    // the graph cache invalidates).
    wb.set(
        "Sheet1",
        addr("B1"),
        CellInput::Formula("=A1+RAND()".into()),
    )
    .unwrap();
    assert!(!wb.graph_cache_is_warm());

    let (changes, closure) = edit_c1(&mut wb, 2.0);
    assert_eq!(wb.graph_builds(), 2);
    assert_eq!(
        closure, 1,
        "B1 is now volatile and must be seeded on every incremental recalc"
    );
    assert!(
        touched(&changes).contains(&"B1".to_string()),
        "{:?}",
        touched(&changes)
    );

    // Flip B1 back to a non-volatile formula. A set-membership bug that
    // inserts on the add path but never evicts the old entry (or the
    // reverse) would get exactly this direction wrong silently.
    wb.set("Sheet1", addr("B1"), CellInput::Formula("=A1+1".into()))
        .unwrap();
    assert!(!wb.graph_cache_is_warm());

    let (_changes, closure) = edit_c1(&mut wb, 3.0);
    assert_eq!(wb.graph_builds(), 3);
    assert_eq!(
        closure, 0,
        "B1 is no longer volatile; the cached set must have evicted it"
    );
}
