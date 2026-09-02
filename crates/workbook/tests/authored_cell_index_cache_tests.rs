//! The authored-cell-index cache (issue #991 fallback design): what it
//! reuses, what invalidates it, and that reusing it can never change an
//! answer.
//!
//! # Why this is a separate suite
//!
//! This cache sits beside the dependency-graph cache and the spill-anchor
//! cache on [`Workbook`], invalidated on its own, genuinely separate
//! schedule — see the `authored_cell_index_cache` module docs. In
//! particular, recalc's own value write-back (`apply_changes`) never
//! invalidates it (it only ever rewrites a cell that already carries formula
//! text, so it can change a value but never adds or removes an authored
//! entry), and `move_sheet` never invalidates it either (reordering tabs
//! changes neither the folded-name keys nor any sheet's authored cells) —
//! both genuinely different from the dependency-graph cache's own schedule.
//!
//! # The rule under test
//!
//! The index is a function of exactly which addresses are authored (formula
//! or literal) on each folded sheet name — never of any cell's *value*. So a
//! write invalidates it exactly when it adds or removes an authored entry
//! (`Workbook::set`'s `introduces_new_cell`, `Workbook::clear` actually
//! removing something) or could have changed the folded-name key space or
//! handed out unobserved write access (`sheets_mut`, `sheet_mut`,
//! `insert_sheet`, `remove_sheet`, `rename_sheet`) — never on value alone.
//!
//! `authored_index_builds()`/`authored_index_cache_is_warm()` are this
//! cache's exact-count instrumentation, mirroring
//! `anchor_builds()`/`anchor_cache_is_warm()` (`spill_anchor_cache_tests.rs`'s
//! own idiom, itself mirroring `graph_cache_tests.rs`'s) — wall clock is too
//! machine-dependent to assert a "was it rebuilt?" question, so the tests
//! below pin build counts instead.
//!
//! Only [`Workbook::recalc_incremental`] ever populates this cache, and even
//! then only lazily: [`Workbook::recalc`] never calls `seed_spill_sensitive`
//! at all, and an incremental recalc whose workbook has no range precedent
//! anywhere builds nothing either (issue #927 follow-up, pinned separately by
//! `authored_cell_index_tests.rs`) — so every fixture below includes at least
//! one range-precedent formula.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).expect("valid A1")
}

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

/// One sheet: `A1`/`A2` are literals, `B1 = SUM(A1:A2)` is a range-precedent
/// formula — the only kind of precedent that ever examines (and so can build)
/// the authored-cell index (see the module docs above).
fn wb_with_range_formula() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", addr("A2"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.set("S", addr("B1"), CellInput::Formula("=SUM(A1:A2)".into()))
        .unwrap();
    wb
}

fn incremental_on_a1(wb: &mut Workbook) {
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("A1"))]);
}

/// Runs `edited` through both a full recalc (on a clone) and an incremental
/// recalc (on `wb` itself), and asserts the two grids are byte-identical
/// canonical JSON — the overall correctness gate this cache's invalidation
/// contract must never compromise.
fn assert_incremental_equals_full(wb: &mut Workbook, edited: &[(&str, &str)]) {
    let mut full = wb.clone();
    full.recalc(&ctx());
    let owned: Vec<(String, Address)> = edited
        .iter()
        .map(|(s, a)| ((*s).to_string(), addr(a)))
        .collect();
    wb.recalc_incremental(&ctx(), &owned);
    assert_eq!(
        wb.to_json().unwrap(),
        full.to_json().unwrap(),
        "incremental and full grids diverged"
    );
}

// ---------------------------------------------------------------------------
// Exact build counts: what the cache reuses
// ---------------------------------------------------------------------------

#[test]
fn a_cold_workbook_builds_the_index_on_first_incremental_recalc() {
    let mut wb = wb_with_range_formula();
    assert_eq!(wb.authored_index_builds(), 0);
    assert!(!wb.authored_index_cache_is_warm());
    incremental_on_a1(&mut wb);
    assert_eq!(
        wb.authored_index_builds(),
        1,
        "the first incremental recalc must build the index exactly once"
    );
    assert!(wb.authored_index_cache_is_warm());
}

#[test]
fn repeating_an_unchanged_incremental_recalc_never_rebuilds() {
    let mut wb = wb_with_range_formula();
    incremental_on_a1(&mut wb);
    let builds = wb.authored_index_builds();
    for _ in 0..20 {
        incremental_on_a1(&mut wb);
    }
    assert_eq!(
        wb.authored_index_builds(),
        builds,
        "recalculating an unchanged workbook must not rebuild the index"
    );
}

/// The negative test that's the whole point of this cache: an edit that
/// changes neither the authored-cell set nor any sheet structure leaves the
/// cache warm, and the incremental result is still byte-identical to a full
/// recalc — reuse must never be observable in the answer.
#[test]
fn an_edit_touching_neither_the_authored_set_nor_sheet_structure_leaves_the_cache_warm() {
    let mut wb = wb_with_range_formula();
    wb.set("S", addr("Z1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    incremental_on_a1(&mut wb);
    let builds_before = wb.authored_index_builds();
    assert!(wb.authored_index_cache_is_warm());

    // An ordinary literal-over-literal edit: no cell is added or removed, no
    // sheet structure changes.
    wb.set("S", addr("Z1"), CellInput::Literal(Value::Number(99.0)))
        .unwrap();
    assert_incremental_equals_full(&mut wb, &[("S", "Z1")]);

    assert_eq!(
        wb.authored_index_builds(),
        builds_before,
        "an edit touching neither the authored set nor sheet structure must \
         not rebuild the index"
    );
    assert!(wb.authored_index_cache_is_warm());
}

// ---------------------------------------------------------------------------
// One rebuild test per real invalidation condition
// ---------------------------------------------------------------------------

#[test]
fn a_new_authored_cell_rebuilds() {
    let mut wb = wb_with_range_formula();
    incremental_on_a1(&mut wb);
    let builds_before = wb.authored_index_builds();
    assert!(wb.authored_index_cache_is_warm());

    // A genuinely new authored cell (`introduces_new_cell`), not an overwrite.
    wb.set("S", addr("Z1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    assert!(
        !wb.authored_index_cache_is_warm(),
        "a new authored cell must invalidate on the write itself"
    );
    assert_incremental_equals_full(&mut wb, &[("S", "Z1")]);
    assert!(
        wb.authored_index_builds() > builds_before,
        "a new authored cell must have forced a rebuild"
    );
}

#[test]
fn a_removed_authored_cell_rebuilds() {
    let mut wb = wb_with_range_formula();
    wb.set("S", addr("Z1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    incremental_on_a1(&mut wb);
    let builds_before = wb.authored_index_builds();
    assert!(wb.authored_index_cache_is_warm());

    wb.clear("S", addr("Z1"));
    assert!(
        !wb.authored_index_cache_is_warm(),
        "removing an authored cell must invalidate on the write itself"
    );
    incremental_on_a1(&mut wb);
    assert!(
        wb.authored_index_builds() > builds_before,
        "removing an authored cell must have forced a rebuild"
    );
}

#[test]
fn every_sheet_structure_operation_that_can_add_or_remove_or_rekey_an_authored_cell_rebuilds() {
    for op in ["add", "rename", "remove"] {
        let mut wb = wb_with_range_formula();
        wb.add_sheet(Worksheet::new("T")).unwrap(); // a spare sheet to remove
        incremental_on_a1(&mut wb);
        assert!(wb.authored_index_cache_is_warm(), "{op}: warm before");

        match op {
            "add" => {
                wb.add_sheet(Worksheet::new("U")).unwrap();
            }
            "rename" => {
                wb.rename_sheet("S", "S2").unwrap();
            }
            "remove" => {
                wb.remove_sheet("T").unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            !wb.authored_index_cache_is_warm(),
            "{op} sheet must invalidate the authored-cell-index cache"
        );
    }
}

/// `move_sheet` is deliberately **not** among the invalidation sites: it
/// changes neither the folded-name key space nor any sheet's authored cells
/// (only tab *order*, which the index does not key on at all) — the same
/// exemption the spill-anchor cache documents for itself.
#[test]
fn moving_a_sheet_does_not_invalidate_the_authored_index_cache() {
    let mut wb = wb_with_range_formula();
    wb.add_sheet(Worksheet::new("T")).unwrap();
    incremental_on_a1(&mut wb);
    assert!(wb.authored_index_cache_is_warm());
    let builds_before = wb.authored_index_builds();

    wb.move_sheet(0, 1).unwrap();

    assert!(
        wb.authored_index_cache_is_warm(),
        "moving a sheet must not invalidate the authored-cell-index cache"
    );
    assert_eq!(wb.authored_index_builds(), builds_before);
}

#[test]
fn every_mutable_accessor_invalidates_on_the_borrow() {
    // What a caller does with a `&mut` into the workbook's interior is
    // unobservable from inside, so handing one out has to be treated as a
    // structural change whether or not the caller makes one.
    let mut wb = wb_with_range_formula();
    incremental_on_a1(&mut wb);
    assert!(wb.authored_index_cache_is_warm());
    let _ = wb.sheets_mut();
    assert!(
        !wb.authored_index_cache_is_warm(),
        "sheets_mut must invalidate on the borrow"
    );

    let mut wb = wb_with_range_formula();
    incremental_on_a1(&mut wb);
    assert!(wb.authored_index_cache_is_warm());
    let _ = wb.sheet_mut("S");
    assert!(
        !wb.authored_index_cache_is_warm(),
        "sheet_mut must invalidate on the borrow"
    );
}

// ---------------------------------------------------------------------------
// Cold-loaded documents
// ---------------------------------------------------------------------------

/// A workbook loaded from JSON, never previously recalculated in this
/// process, still builds the index correctly on its first incremental call.
#[test]
fn a_workbook_loaded_from_json_builds_the_index_correctly_on_its_first_incremental_call() {
    let json = wb_with_range_formula().to_json().unwrap();
    let mut wb = Workbook::from_json(json.as_bytes()).unwrap();
    assert!(!wb.authored_index_cache_is_warm());

    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    assert_incremental_equals_full(&mut wb, &[("S", "A1")]);

    assert!(wb.authored_index_cache_is_warm());
    assert_eq!(wb.authored_index_builds(), 1);
    assert_eq!(
        wb.get("S", addr("B1")).unwrap().value(),
        &Value::Number(7.0)
    );
}
