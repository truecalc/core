//! The spill-anchor-rectangle cache (issue #984): what it reuses, what
//! invalidates it, and that reusing it can never change an answer.
//!
//! # Why this is a separate suite from `graph_cache_tests.rs`
//!
//! The two caches sit beside each other on [`Workbook`] but are invalidated
//! on genuinely different schedules — see the `spill_anchor_cache` module
//! docs. In particular, recalc's own value write-back (`apply_changes`)
//! deliberately does **not** invalidate the dependency-graph cache (it
//! preserves formula text, adding no node or edge) but **must** invalidate
//! this one whenever it places, resizes, or removes a spill. A workbook with
//! zero formulas can also flip a cell's array-ness through an ordinary
//! literal write (`CellInput::Literal(Value::Array(..))` is a supported input
//! shape), which the graph cache's rules cannot see at all.
//!
//! # The rule under test
//!
//! The anchor map is a function of exactly which authored cells currently
//! hold a `Value::Array`. So a write invalidates it exactly when the old
//! value was an array, or the new value is — never on formula-shape alone,
//! and never left un-invalidated just because the graph cache stayed warm.
//!
//! `anchor_builds()`/`anchor_cache_is_warm()` are this cache's exact-count
//! instrumentation, mirroring `graph_builds()`/`graph_cache_is_warm()`
//! (`graph_cache_tests.rs`'s own idiom) — wall clock is too
//! machine-dependent to assert a "was it rebuilt?" question, so the tests
//! below pin build counts instead.
//!
//! Only [`Workbook::recalc_incremental`] ever populates this cache:
//! [`Workbook::recalc`] evaluates every formula cell directly and never runs
//! the spill-widen loop that reads it, so a full recalc can *invalidate* the
//! entry (through the shared `apply_changes` write-back) but never warms it.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).expect("valid A1")
}

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn incremental_on_a1(wb: &mut Workbook) {
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("A1"))]);
}

/// One sheet: `A1` is a plain literal controlling how many elements `B1`'s
/// `SEQUENCE` spills into. `SEQUENCE(1, 1)` collapses to a scalar (schema spec
/// §6 — a 1×1 array is never stored as one), so `A1 == 1` means "`B1` does not
/// spill"; `A1 >= 2` means it does, across `B1..` on row 1. Editing `A1` is
/// therefore a literal write on a *precedent*, not a formula-text change —
/// exactly the shape `apply_changes`'s write-back to `B1` exercises.
fn wb_with_variable_spill() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set(
        "S",
        addr("B1"),
        CellInput::Formula("=SEQUENCE(1, A1)".into()),
    )
    .unwrap();
    wb
}

fn set_a1(wb: &mut Workbook, n: f64) {
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(n)))
        .unwrap();
}

// ---------------------------------------------------------------------------
// Exact build counts: what the cache reuses
// ---------------------------------------------------------------------------

#[test]
fn a_cold_workbook_builds_the_anchor_map_on_first_incremental_recalc() {
    let mut wb = wb_with_variable_spill();
    assert_eq!(wb.anchor_builds(), 0);
    assert!(!wb.anchor_cache_is_warm());
    incremental_on_a1(&mut wb);
    assert!(
        wb.anchor_builds() >= 1,
        "the first incremental recalc must build the anchor map at least once"
    );
    assert!(wb.anchor_cache_is_warm());
}

#[test]
fn repeating_an_unchanged_incremental_recalc_never_rebuilds() {
    let mut wb = wb_with_variable_spill();
    incremental_on_a1(&mut wb);
    let builds = wb.anchor_builds();
    for _ in 0..20 {
        incremental_on_a1(&mut wb);
    }
    assert_eq!(
        wb.anchor_builds(),
        builds,
        "recalculating an unchanged workbook must not rebuild the anchor map"
    );
}

// ---------------------------------------------------------------------------
// The four correctness scenarios the issue asks for
// ---------------------------------------------------------------------------

#[test]
fn a_cell_gaining_an_array_value_rebuilds_and_the_new_spill_is_reflected() {
    let mut wb = wb_with_variable_spill();
    incremental_on_a1(&mut wb); // A1 == 1: B1 has no spill yet.
    assert!(matches!(
        wb.get("S", addr("B1")).unwrap().value(),
        Value::Number(_)
    ));

    set_a1(&mut wb, 3.0); // B1 now spills across B1:D1.
    let builds_before = wb.anchor_builds();
    incremental_on_a1(&mut wb);

    assert!(
        wb.anchor_builds() > builds_before,
        "a cell gaining an array value must rebuild the anchor cache"
    );
    assert!(matches!(
        wb.get("S", addr("B1")).unwrap().value(),
        Value::Array(_)
    ));
    assert_eq!(
        wb.resolved("S", addr("C1")).unwrap().anchor,
        Some(addr("B1"))
    );
    assert_eq!(
        wb.resolved("S", addr("D1")).unwrap().anchor,
        Some(addr("B1"))
    );
}

#[test]
fn a_cell_losing_its_array_value_rebuilds_and_the_cleared_spill_is_reflected() {
    let mut wb = wb_with_variable_spill();
    set_a1(&mut wb, 3.0);
    incremental_on_a1(&mut wb); // B1 spills across B1:D1.
    assert!(matches!(
        wb.get("S", addr("B1")).unwrap().value(),
        Value::Array(_)
    ));

    set_a1(&mut wb, 1.0); // B1 collapses back to a scalar.
    let builds_before = wb.anchor_builds();
    incremental_on_a1(&mut wb);

    assert!(
        wb.anchor_builds() > builds_before,
        "a cell losing its array value must rebuild the anchor cache"
    );
    assert!(matches!(
        wb.get("S", addr("B1")).unwrap().value(),
        Value::Number(_)
    ));
    assert!(
        wb.resolved("S", addr("C1")).is_none(),
        "the vacated spill cell must no longer resolve as spilled"
    );
}

#[test]
fn a_spills_footprint_resizing_rebuilds_and_the_new_footprint_is_reflected() {
    let mut wb = wb_with_variable_spill();
    set_a1(&mut wb, 2.0);
    incremental_on_a1(&mut wb); // B1 spills across B1:C1 only.
    assert_eq!(
        wb.resolved("S", addr("C1")).unwrap().anchor,
        Some(addr("B1"))
    );
    assert!(wb.resolved("S", addr("D1")).is_none());

    set_a1(&mut wb, 4.0); // B1 widens to B1:E1.
    let builds_before = wb.anchor_builds();
    incremental_on_a1(&mut wb);

    assert!(
        wb.anchor_builds() > builds_before,
        "a resized spill footprint must rebuild the anchor cache"
    );
    assert_eq!(
        wb.resolved("S", addr("D1")).unwrap().anchor,
        Some(addr("B1"))
    );
    assert_eq!(
        wb.resolved("S", addr("E1")).unwrap().anchor,
        Some(addr("B1"))
    );
}

#[test]
fn an_edit_touching_neither_formulas_nor_array_ness_does_not_rebuild() {
    let mut wb = wb_with_variable_spill();
    set_a1(&mut wb, 3.0); // a real spill exists elsewhere on the sheet.
    wb.set("S", addr("Z1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    incremental_on_a1(&mut wb);
    let builds_before = wb.anchor_builds();
    assert!(wb.anchor_cache_is_warm());

    // An ordinary literal-over-literal edit, unrelated to B1's spill: not a
    // formula, and neither its old nor new value is an array.
    wb.set("S", addr("Z1"), CellInput::Literal(Value::Number(99.0)))
        .unwrap();
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("Z1"))]);

    assert_eq!(
        wb.anchor_builds(),
        builds_before,
        "an edit touching neither formulas nor array-ness must not rebuild the anchor cache"
    );
    assert!(wb.anchor_cache_is_warm());
}

// ---------------------------------------------------------------------------
// The two schedules are genuinely separate — the load-bearing distinction
// ---------------------------------------------------------------------------

#[test]
fn a_literal_array_write_invalidates_the_anchor_cache_while_the_graph_cache_stays_warm() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    // No formulas anywhere in this workbook, so there is nothing to
    // recompute — but the incremental path still pre-warms both caches.
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("A1"))]);
    assert!(wb.graph_cache_is_warm());
    assert!(wb.anchor_cache_is_warm());

    // A literal write of an array value over a previously-scalar cell in a
    // table-free, formula-free workbook: no `GraphCache` invalidation clause
    // applies (no formula node added or removed, no table declared), so the
    // graph cache must stay warm. The anchor cache must invalidate anyway —
    // this is exactly the write the graph cache's own schedule cannot see.
    let arr = Value::Array(vec![vec![Value::Number(1.0), Value::Number(2.0)]]);
    wb.set("S", addr("A1"), CellInput::Literal(arr)).unwrap();

    assert!(
        wb.graph_cache_is_warm(),
        "a literal write over a literal, table-free cell adds no graph node"
    );
    assert!(
        !wb.anchor_cache_is_warm(),
        "an array-shaped literal write must invalidate the anchor cache even \
         though the graph cache stays warm"
    );
}

#[test]
fn a_workbook_loaded_from_json_with_a_pre_existing_spill_builds_the_anchor_map_correctly() {
    // A pre-existing spill authored entirely through a prior process, not
    // this process's own formula evaluation: recalc it once to place the
    // spill on the grid, round-trip through JSON, and confirm a freshly
    // constructed `Workbook` — whose cache starts cold by construction (see
    // the `spill_anchor_cache` module docs) — builds a correct anchor map
    // from the loaded grid on its first incremental recalc.
    let mut wb = wb_with_variable_spill();
    set_a1(&mut wb, 3.0);
    wb.recalc(&ctx()); // B1 spills across B1:D1, stored on the grid.
    let json = wb.to_json().expect("a recalculated workbook serializes");

    let mut loaded = Workbook::from_json(json.as_bytes()).expect("round-trips");
    assert_eq!(
        loaded.anchor_builds(),
        0,
        "a freshly loaded workbook starts with a cold anchor cache regardless of construction path"
    );
    assert!(!loaded.anchor_cache_is_warm());

    // An edit unrelated to B1's spill still forces the incremental path to
    // build the anchor map from the grid as loaded — there is no formula
    // evaluation history in this process for B1's spill to have come from.
    loaded
        .set("S", addr("Z1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    loaded.recalc_incremental(&ctx(), &[("S".to_owned(), addr("Z1"))]);

    assert!(loaded.anchor_cache_is_warm());
    assert_eq!(
        loaded.resolved("S", addr("C1")).unwrap().anchor,
        Some(addr("B1")),
        "the pre-existing spill loaded from JSON must be reflected in the anchor cache"
    );
    assert_eq!(
        loaded.resolved("S", addr("D1")).unwrap().anchor,
        Some(addr("B1"))
    );
}

#[test]
fn a_full_recalc_that_changes_a_spill_leaves_no_stale_entry_for_the_next_incremental_call() {
    let mut wb = wb_with_variable_spill();
    incremental_on_a1(&mut wb); // A1 == 1: warms the anchor cache with no spills.
    assert!(wb.anchor_cache_is_warm());

    // Edit A1 to make B1 spill, then recalc via the FULL path — `recalc`
    // itself never touches the anchor cache (only the incremental widen loop
    // does), but its value write-back goes through the same `apply_changes`
    // invalidation check, so the stale "no spills" entry must not survive it.
    set_a1(&mut wb, 3.0);
    wb.recalc(&ctx());
    assert!(
        !wb.anchor_cache_is_warm(),
        "recalc's own write-back must invalidate the anchor cache exactly \
         like recalc_incremental's does"
    );

    // The next incremental recalc must rebuild fresh and see the new spill,
    // not reuse whatever was cached before the full recalc ran.
    incremental_on_a1(&mut wb);
    assert!(matches!(
        wb.get("S", addr("B1")).unwrap().value(),
        Value::Array(_)
    ));
    assert_eq!(
        wb.resolved("S", addr("D1")).unwrap().anchor,
        Some(addr("B1"))
    );
}
