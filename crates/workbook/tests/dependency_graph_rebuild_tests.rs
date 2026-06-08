//! Graph-rebuild equivalence after arbitrary edit sequences (issue #534
//! acceptance criterion).
//!
//! The dependency graph is a pure function of the workbook's formulas, sheet
//! names, and named-range targets. These tests apply edit sequences — the
//! set/clear/rename/retarget operations the rebuild rules cover — through the
//! public value-object API, then assert that:
//!
//! 1. a graph built from the edited workbook equals a graph built from a fresh
//!    workbook constructed to the same final state (rebuild is a deterministic
//!    function of state, the property P3.4's maintained graph is checked
//!    against), and
//! 2. each individual edit is reflected in the rebuilt graph (set adds edges,
//!    clear removes them, rename re-keys cross-sheet edges, retargeting a name
//!    moves its indirection).
//!
//! Recalc (P3.3) is out of scope here; these assert the *graph*, not values.

use truecalc_workbook::{
    Address, Cell, CellRef, DependencyGraph, EngineFlavor, NamedRange, Value, Workbook, Worksheet,
};

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).unwrap()
}

fn cref(sheet: &str, a1: &str) -> CellRef {
    CellRef {
        sheet: sheet.to_string(),
        addr: addr(a1),
    }
}

fn set_formula(wb: &mut Workbook, sheet: &str, a1: &str, formula: &str) {
    wb.sheet_mut(sheet)
        .unwrap()
        .set(addr(a1), Cell::with_formula(formula, Value::Empty));
}

fn set_num(wb: &mut Workbook, sheet: &str, a1: &str, n: f64) {
    wb.sheet_mut(sheet)
        .unwrap()
        .set(addr(a1), Cell::literal(Value::Number(n)).unwrap());
}

fn two_sheets() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb.add_sheet(Worksheet::new("Sheet2")).unwrap();
    wb
}

#[test]
fn rebuild_equals_fresh_after_edit_sequence() {
    // Build a workbook by an edit sequence.
    let mut edited = two_sheets();
    set_num(&mut edited, "Sheet1", "A1", 1.0);
    set_formula(&mut edited, "Sheet1", "B1", "=A1+1");
    set_formula(&mut edited, "Sheet1", "C1", "=SUM(A1:B1)");
    set_formula(&mut edited, "Sheet2", "A1", "=Sheet1!C1*2");
    // Overwrite B1 with a different formula.
    set_formula(&mut edited, "Sheet1", "B1", "=A1*10");
    // Clear C1, then re-add it differently.
    edited.sheet_mut("Sheet1").unwrap().clear(addr("C1"));
    set_formula(&mut edited, "Sheet1", "C1", "=A1-B1");

    // Construct the same *final* state directly.
    let mut fresh = two_sheets();
    set_num(&mut fresh, "Sheet1", "A1", 1.0);
    set_formula(&mut fresh, "Sheet1", "B1", "=A1*10");
    set_formula(&mut fresh, "Sheet1", "C1", "=A1-B1");
    set_formula(&mut fresh, "Sheet2", "A1", "=Sheet1!C1*2");

    assert_eq!(
        DependencyGraph::build(&edited),
        DependencyGraph::build(&fresh),
        "graph must be a deterministic function of final workbook state"
    );
}

#[test]
fn build_is_deterministic_for_equal_workbooks() {
    let mut wb = two_sheets();
    set_formula(&mut wb, "Sheet1", "A1", "=SUM(Sheet2!A1:A9, B1)");
    set_formula(&mut wb, "Sheet2", "A1", "=B1");
    assert_eq!(DependencyGraph::build(&wb), DependencyGraph::build(&wb));
}

#[test]
fn set_then_clear_returns_to_empty_graph() {
    let mut wb = two_sheets();
    let empty = DependencyGraph::build(&wb);
    set_formula(&mut wb, "Sheet1", "B1", "=A1");
    assert_ne!(DependencyGraph::build(&wb), empty);
    wb.sheet_mut("Sheet1").unwrap().clear(addr("B1"));
    assert_eq!(
        DependencyGraph::build(&wb),
        empty,
        "clearing the only formula returns the graph to empty"
    );
}

#[test]
fn rename_sheet_rekeys_cross_sheet_edges() {
    let mut wb = two_sheets();
    set_num(&mut wb, "Sheet2", "A1", 5.0);
    set_formula(&mut wb, "Sheet1", "A1", "=Sheet2!A1");

    let before = DependencyGraph::build(&wb);
    assert!(before
        .direct_dependents_of(&cref("sheet2", "A1"))
        .contains(&cref("sheet1", "A1")));

    // Rename Sheet2 → Data. The formula text still says "Sheet2!A1", which now
    // dangles — the rebuild rule for rename surfaces this (Sheets would rewrite
    // the formula on rename, which is structural-edit territory deferred to a
    // post-v1 issue; here the graph just reflects the new state honestly).
    wb.rename_sheet("Sheet2", "Data").unwrap();
    let after = DependencyGraph::build(&wb);
    // The old folded-name edge is gone.
    assert!(after.direct_dependents_of(&cref("sheet2", "A1")).is_empty());
    assert_ne!(before, after);
}

#[test]
fn retarget_name_moves_indirection() {
    let mut wb = two_sheets();
    wb.names_mut().push(NamedRange {
        name: "Target".to_string(),
        r#ref: "Sheet1!A1".to_string(),
    });
    set_formula(&mut wb, "Sheet1", "B1", "=Target+1");

    let g1 = DependencyGraph::build(&wb);
    assert!(g1
        .direct_dependents_of(&cref("sheet1", "A1"))
        .contains(&cref("sheet1", "B1")));
    assert!(g1.direct_dependents_of(&cref("sheet1", "A2")).is_empty());

    // Retarget the name A1 → A2 (the P3.4 named-range CRUD; done here directly
    // on the value object to exercise the graph's rebuild rule for it).
    wb.names_mut()[0].r#ref = "Sheet1!A2".to_string();
    let g2 = DependencyGraph::build(&wb);

    // Dependents of A1 no longer include B1; dependents of A2 now do. B1's
    // formula text is unchanged — only the name's target moved.
    assert!(g2.direct_dependents_of(&cref("sheet1", "A1")).is_empty());
    assert!(g2
        .direct_dependents_of(&cref("sheet1", "A2"))
        .contains(&cref("sheet1", "B1")));
    // The name still has B1 as a dependent (retargeting dirties it).
    assert!(g2
        .name_dependents_of("Target")
        .contains(&cref("sheet1", "B1")));
    assert_ne!(g1, g2);
}

#[test]
fn idempotent_set_of_same_formula_is_a_no_op() {
    let mut wb = two_sheets();
    set_formula(&mut wb, "Sheet1", "B1", "=A1+A2");
    let g1 = DependencyGraph::build(&wb);
    set_formula(&mut wb, "Sheet1", "B1", "=A1+A2"); // same text again
    let g2 = DependencyGraph::build(&wb);
    assert_eq!(g1, g2);
}

#[test]
fn many_random_like_edits_then_rebuild_matches_replay() {
    // A longer mixed sequence; rebuild from the live workbook must equal a
    // rebuild from a workbook replayed to the identical final state.
    let mut wb = two_sheets();
    let edits: &[(&str, &str, Option<&str>)] = &[
        ("Sheet1", "A1", Some("=B1+C1")),
        ("Sheet1", "B1", Some("=10")),
        ("Sheet1", "C1", Some("=B1*2")),
        ("Sheet2", "A1", Some("=Sheet1!A1")),
        ("Sheet1", "A1", None),                // clear A1
        ("Sheet1", "A1", Some("=SUM(B1:C1)")), // re-add A1 as a range reader
        ("Sheet1", "C1", Some("=B1+5")),       // overwrite C1
        ("Sheet2", "B2", Some("=Sheet1!C1")),
    ];
    for (sheet, a1, formula) in edits {
        match formula {
            Some(f) => set_formula(&mut wb, sheet, a1, f),
            None => {
                wb.sheet_mut(sheet).unwrap().clear(addr(a1));
            }
        }
    }
    let live = DependencyGraph::build(&wb);

    // Final state, written directly.
    let mut replay = two_sheets();
    set_formula(&mut replay, "Sheet1", "A1", "=SUM(B1:C1)");
    set_formula(&mut replay, "Sheet1", "B1", "=10");
    set_formula(&mut replay, "Sheet1", "C1", "=B1+5");
    set_formula(&mut replay, "Sheet2", "A1", "=Sheet1!A1");
    set_formula(&mut replay, "Sheet2", "B2", "=Sheet1!C1");

    assert_eq!(live, DependencyGraph::build(&replay));
}
