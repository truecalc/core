//! Dependency-graph construction, resolution, and cycle/topology tests
//! (plan item 3.2, issue #534).
//!
//! Verifies that precedents are derived from each formula via core's
//! `extract_refs` (P1.3), that cross-sheet and named references resolve to
//! concrete graph nodes, that ranges are compressed to a single node, and that
//! cycles and topological order are reported correctly. The graph-rebuild
//! equivalence property (the issue's acceptance criterion) lives in
//! `dependency_graph_rebuild_tests.rs`.

use std::collections::{BTreeSet, HashSet};

use truecalc_workbook::{
    Address, Cell, CellRef, DependencyGraph, EngineFlavor, NameTarget, NamedRange, Precedent,
    Value, Workbook, Worksheet,
};

/// A Sheets workbook with one sheet named `Sheet1`.
fn wb_one_sheet() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb
}

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).unwrap()
}

/// Sets a formula cell `a1 = formula` on sheet `sheet` (value empty until
/// recalc, which is P3.3).
fn set_formula(wb: &mut Workbook, sheet: &str, a1: &str, formula: &str) {
    wb.sheet_mut(sheet)
        .unwrap()
        .set(addr(a1), Cell::with_formula(formula, Value::Empty));
}

/// Sets a literal number cell.
fn set_num(wb: &mut Workbook, sheet: &str, a1: &str, n: f64) {
    wb.sheet_mut(sheet)
        .unwrap()
        .set(addr(a1), Cell::literal(Value::Number(n)).unwrap());
}

fn cref(sheet: &str, a1: &str) -> CellRef {
    CellRef {
        sheet: sheet.to_string(),
        addr: addr(a1),
    }
}

#[test]
fn single_cell_precedent_from_extract_refs() {
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "A1", 10.0);
    set_formula(&mut wb, "Sheet1", "B1", "=A1+1");

    let g = DependencyGraph::build(&wb);
    let precs = g.precedents_of(&cref("sheet1", "B1")).unwrap();
    assert_eq!(precs, &[Precedent::Cell(cref("sheet1", "A1"))]);

    let deps = g.direct_dependents_of(&cref("sheet1", "A1"));
    assert!(deps.contains(&cref("sheet1", "B1")));
}

#[test]
fn literal_cells_are_not_formula_nodes() {
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "A1", 5.0);
    let g = DependencyGraph::build(&wb);
    assert!(!g.is_formula(&cref("sheet1", "A1")));
    assert!(g.precedents_of(&cref("sheet1", "A1")).is_none());
    assert_eq!(g.formula_cells().count(), 0);
}

#[test]
fn duplicate_refs_in_one_formula_are_deduplicated() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "B1", "=A1+A1+A1");
    let g = DependencyGraph::build(&wb);
    let precs = g.precedents_of(&cref("sheet1", "B1")).unwrap();
    assert_eq!(precs, &[Precedent::Cell(cref("sheet1", "A1"))]);
}

#[test]
fn refs_found_in_every_argument_position() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "D1", "=SUM(A1, B1, C1)");
    let g = DependencyGraph::build(&wb);
    let precs: HashSet<_> = g
        .precedents_of(&cref("sheet1", "D1"))
        .unwrap()
        .iter()
        .cloned()
        .collect();
    assert_eq!(
        precs,
        HashSet::from([
            Precedent::Cell(cref("sheet1", "A1")),
            Precedent::Cell(cref("sheet1", "B1")),
            Precedent::Cell(cref("sheet1", "C1")),
        ])
    );
}

#[test]
fn cross_sheet_ref_resolves_to_named_sheet() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb.add_sheet(Worksheet::new("Data")).unwrap();
    set_num(&mut wb, "Data", "B2", 7.0);
    set_formula(&mut wb, "Sheet1", "A1", "=Data!B2*2");

    let g = DependencyGraph::build(&wb);
    let precs = g.precedents_of(&cref("sheet1", "A1")).unwrap();
    assert_eq!(precs, &[Precedent::Cell(cref("data", "B2"))]);

    assert!(g
        .direct_dependents_of(&cref("data", "B2"))
        .contains(&cref("sheet1", "A1")));
}

#[test]
fn bare_ref_resolves_against_formulas_own_sheet() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb.add_sheet(Worksheet::new("Sheet2")).unwrap();
    set_formula(&mut wb, "Sheet1", "A1", "=B1");
    set_formula(&mut wb, "Sheet2", "A1", "=B1");

    let g = DependencyGraph::build(&wb);
    assert_eq!(
        g.precedents_of(&cref("sheet1", "A1")).unwrap(),
        &[Precedent::Cell(cref("sheet1", "B1"))]
    );
    assert_eq!(
        g.precedents_of(&cref("sheet2", "A1")).unwrap(),
        &[Precedent::Cell(cref("sheet2", "B1"))]
    );
}

#[test]
fn unknown_sheet_is_unresolved_no_edge() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "A1", "=Ghost!B2");
    let g = DependencyGraph::build(&wb);
    let precs = g.precedents_of(&cref("sheet1", "A1")).unwrap();
    assert_eq!(precs.len(), 1);
    assert!(matches!(&precs[0], Precedent::Unresolved(_)));
    assert!(g.direct_dependents_of(&cref("ghost", "B2")).is_empty());
}

#[test]
fn unresolved_dedupes_regardless_of_dollar_anchors() {
    // Two references to the same missing-sheet target, one plain and one
    // $-anchored, must dedupe to a single Unresolved precedent — `$` is a
    // display marker only, never an identity difference (issue #708).
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "A1", "=Ghost!B2+Ghost!$B$2");
    let g = DependencyGraph::build(&wb);
    let precs = g.precedents_of(&cref("sheet1", "A1")).unwrap();
    assert_eq!(precs.len(), 1);
    assert!(matches!(&precs[0], Precedent::Unresolved(_)));
}

#[test]
fn unparseable_formula_is_unresolved() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "A1", "=SUM(");
    let g = DependencyGraph::build(&wb);
    let precs = g.precedents_of(&cref("sheet1", "A1")).unwrap();
    assert_eq!(precs.len(), 1);
    assert!(matches!(&precs[0], Precedent::Unresolved(_)));
}

#[test]
fn range_is_one_compressed_node() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "B1", "=SUM(A1:A100)");
    let g = DependencyGraph::build(&wb);
    let precs = g.precedents_of(&cref("sheet1", "B1")).unwrap();
    assert_eq!(precs.len(), 1, "a range is a single precedent node");
    match &precs[0] {
        Precedent::Range(r) => {
            assert_eq!(r.start, addr("A1"));
            assert_eq!(r.end, addr("A100"));
            assert_eq!(r.sheet, "sheet1");
        }
        other => panic!("expected a range precedent, got {other:?}"),
    }

    assert!(g
        .direct_dependents_of(&cref("sheet1", "A50"))
        .contains(&cref("sheet1", "B1")));
    assert!(g.direct_dependents_of(&cref("sheet1", "A101")).is_empty());
    assert!(g.direct_dependents_of(&cref("sheet1", "B5")).is_empty());
}

#[test]
fn huge_range_does_not_explode_edges() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "B1", "=SUM(A1:A1000000)");
    let g = DependencyGraph::build(&wb);
    let precs = g.precedents_of(&cref("sheet1", "B1")).unwrap();
    assert_eq!(precs.len(), 1);
    assert!(matches!(&precs[0], Precedent::Range(_)));
    assert!(g
        .direct_dependents_of(&cref("sheet1", "A999999"))
        .contains(&cref("sheet1", "B1")));
}

#[test]
fn named_range_indirection() {
    let mut wb = wb_one_sheet();
    wb.names_mut().push(NamedRange {
        name: "TaxRate".to_string(),
        r#ref: "Sheet1!C1".to_string(),
    });
    set_num(&mut wb, "Sheet1", "C1", 0.2);
    set_formula(&mut wb, "Sheet1", "A1", "=B1*TaxRate");

    let g = DependencyGraph::build(&wb);
    let precs: HashSet<_> = g
        .precedents_of(&cref("sheet1", "A1"))
        .unwrap()
        .iter()
        .cloned()
        .collect();
    assert!(precs.contains(&Precedent::Name("taxrate".to_string())));
    assert!(precs.contains(&Precedent::Cell(cref("sheet1", "B1"))));

    assert!(g
        .name_dependents_of("TaxRate")
        .contains(&cref("sheet1", "A1")));
    assert!(g
        .name_dependents_of("TAXRATE")
        .contains(&cref("sheet1", "A1")));

    assert!(g
        .direct_dependents_of(&cref("sheet1", "C1"))
        .contains(&cref("sheet1", "A1")));
}

#[test]
fn named_range_pointing_at_a_range() {
    let mut wb = wb_one_sheet();
    wb.names_mut().push(NamedRange {
        name: "Region".to_string(),
        r#ref: "Sheet1!A1:A10".to_string(),
    });
    set_formula(&mut wb, "Sheet1", "C1", "=SUM(Region)");
    let g = DependencyGraph::build(&wb);
    assert!(g
        .direct_dependents_of(&cref("sheet1", "A5"))
        .contains(&cref("sheet1", "C1")));
    assert!(g.direct_dependents_of(&cref("sheet1", "A11")).is_empty());
}

#[test]
fn undefined_name_is_unresolved_not_a_name_node() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "A1", "=NoSuchName+1");
    let g = DependencyGraph::build(&wb);
    let precs = g.precedents_of(&cref("sheet1", "A1")).unwrap();
    assert_eq!(precs, &[Precedent::Unresolved("NoSuchName".to_string())]);
}

#[test]
fn acyclic_chain_topological_order() {
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "A1", 1.0);
    set_formula(&mut wb, "Sheet1", "B1", "=A1+1");
    set_formula(&mut wb, "Sheet1", "C1", "=B1+1");
    set_formula(&mut wb, "Sheet1", "D1", "=C1+1");

    let g = DependencyGraph::build(&wb);
    let order = g.topological_order().expect("acyclic");
    let pos = |a1: &str| order.iter().position(|c| *c == cref("sheet1", a1)).unwrap();
    assert!(pos("B1") < pos("C1"));
    assert!(pos("C1") < pos("D1"));
    assert!(g.cycle_cells().is_empty());
}

#[test]
fn deep_chain_does_not_overflow() {
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "A1", 1.0);
    let mut prev = addr("A1");
    for row in 2..=5000u32 {
        let here = Address::new(row, 1).unwrap();
        let f = format!("={}", prev.to_a1());
        wb.sheet_mut("Sheet1")
            .unwrap()
            .set(here, Cell::with_formula(f, Value::Empty));
        prev = here;
    }
    let g = DependencyGraph::build(&wb);
    assert!(g.cycle_cells().is_empty());
    let order = g.topological_order().expect("acyclic");
    assert_eq!(order.len(), 4999);
}

#[test]
fn direct_two_cell_cycle_detected() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "A1", "=B1");
    set_formula(&mut wb, "Sheet1", "B1", "=A1");
    let g = DependencyGraph::build(&wb);
    let cyc = g.cycle_cells();
    assert_eq!(
        cyc,
        BTreeSet::from([cref("sheet1", "A1"), cref("sheet1", "B1")])
    );
    assert!(g.topological_order().is_err());
}

#[test]
fn self_reference_is_a_cycle() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "A1", "=A1+1");
    let g = DependencyGraph::build(&wb);
    assert_eq!(g.cycle_cells(), BTreeSet::from([cref("sheet1", "A1")]));
    assert!(g.topological_order().is_err());
}

#[test]
fn cycle_via_range_and_cross_sheet() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S1")).unwrap();
    wb.add_sheet(Worksheet::new("S2")).unwrap();
    set_formula(&mut wb, "S1", "A1", "=SUM(S2!A1:A3)");
    set_formula(&mut wb, "S2", "A2", "=S1!A1");
    let g = DependencyGraph::build(&wb);
    let cyc = g.cycle_cells();
    assert!(cyc.contains(&cref("s1", "A1")));
    assert!(cyc.contains(&cref("s2", "A2")));
}

#[test]
fn cycle_through_named_range() {
    let mut wb = wb_one_sheet();
    wb.names_mut().push(NamedRange {
        name: "Loop".to_string(),
        r#ref: "Sheet1!A1".to_string(),
    });
    set_formula(&mut wb, "Sheet1", "A1", "=B1");
    set_formula(&mut wb, "Sheet1", "B1", "=Loop");
    let g = DependencyGraph::build(&wb);
    let cyc = g.cycle_cells();
    assert!(cyc.contains(&cref("sheet1", "A1")));
    assert!(cyc.contains(&cref("sheet1", "B1")));
}

#[test]
fn formula_reading_only_literals_has_no_topo_edges() {
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "A1", 1.0);
    set_num(&mut wb, "Sheet1", "A2", 2.0);
    set_formula(&mut wb, "Sheet1", "B1", "=A1+A2");
    let g = DependencyGraph::build(&wb);
    let order = g.topological_order().unwrap();
    assert_eq!(order, vec![cref("sheet1", "B1")]);
}

// --- Public traversal primitives -----------------------------------------

#[test]
fn cell_ref_resolve_folds_the_sheet_name_so_the_key_matches() {
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "A1", 10.0);
    set_formula(&mut wb, "Sheet1", "B1", "=A1+1");
    let g = DependencyGraph::build(&wb);

    // The display casing a caller has does not match the graph key directly...
    assert!(g
        .precedents_of(&CellRef {
            sheet: "Sheet1".to_string(),
            addr: addr("B1"),
        })
        .is_none());
    // ... but `resolve` produces the key the graph actually uses, from any
    // casing, and folding an already-folded name is a no-op.
    for spelling in ["Sheet1", "SHEET1", "sheet1"] {
        assert_eq!(
            g.precedents_of(&CellRef::from_display_name(spelling, addr("B1"))),
            Some(&[Precedent::Cell(cref("sheet1", "A1"))][..]),
            "{spelling} should resolve to the graph's key"
        );
    }
}

#[test]
fn name_target_of_reports_a_cell_target() {
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "B1", 3.0);
    wb.names_mut().push(NamedRange {
        name: "Rate".to_string(),
        r#ref: "Sheet1!B1".to_string(),
    });
    set_formula(&mut wb, "Sheet1", "A1", "=Rate*2");
    let g = DependencyGraph::build(&wb);

    // Any casing, matching `name_dependents_of`'s case-insensitivity.
    for spelling in ["Rate", "RATE", "rate"] {
        assert_eq!(
            g.name_target_of(spelling),
            Some(NameTarget::Cell(cref("sheet1", "B1"))),
            "{spelling} should resolve"
        );
    }
}

#[test]
fn name_target_of_reports_a_range_target() {
    let mut wb = wb_one_sheet();
    wb.names_mut().push(NamedRange {
        name: "Block".to_string(),
        r#ref: "Sheet1!A1:B2".to_string(),
    });
    set_formula(&mut wb, "Sheet1", "D1", "=SUM(Block)");
    let g = DependencyGraph::build(&wb);

    match g.name_target_of("Block") {
        Some(NameTarget::Range(r)) => {
            assert_eq!(r.sheet, "sheet1");
            assert_eq!(r.start, addr("A1"));
            assert_eq!(r.end, addr("B2"));
        }
        other => panic!("expected a range target, got {other:?}"),
    }
}

#[test]
fn name_target_of_is_none_for_an_undefined_name() {
    let wb = wb_one_sheet();
    let g = DependencyGraph::build(&wb);
    assert_eq!(g.name_target_of("NoSuchName"), None);
}

#[test]
fn formula_precedent_cells_yields_only_formula_cells() {
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "A1", 1.0); // literal
    set_formula(&mut wb, "Sheet1", "A2", "=A1+1"); // formula
    set_formula(&mut wb, "Sheet1", "D1", "=SUM(A1:A2)");
    let g = DependencyGraph::build(&wb);

    // A cell precedent pointing at a literal yields nothing to walk.
    assert!(g
        .formula_precedent_cells(&Precedent::Cell(cref("sheet1", "A1")))
        .is_empty());
    // A cell precedent pointing at a formula yields it.
    assert_eq!(
        g.formula_precedent_cells(&Precedent::Cell(cref("sheet1", "A2"))),
        vec![cref("sheet1", "A2")]
    );
    // A range yields the formula cells it covers, not every member.
    let range = match &g.precedents_of(&cref("sheet1", "D1")).unwrap()[0] {
        Precedent::Range(r) => Precedent::Range(r.clone()),
        other => panic!("expected a range precedent, got {other:?}"),
    };
    assert_eq!(
        g.formula_precedent_cells(&range),
        vec![cref("sheet1", "A2")]
    );
    // An unresolved precedent yields nothing.
    assert!(g
        .formula_precedent_cells(&Precedent::Unresolved("Nope!A1".to_string()))
        .is_empty());
}

#[test]
fn formula_precedent_cells_follows_a_name_to_its_target() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "B1", "=1+1");
    wb.names_mut().push(NamedRange {
        name: "Rate".to_string(),
        r#ref: "Sheet1!B1".to_string(),
    });
    set_formula(&mut wb, "Sheet1", "A1", "=Rate*2");
    let g = DependencyGraph::build(&wb);

    assert_eq!(
        g.formula_precedent_cells(&Precedent::Name("rate".to_string())),
        vec![cref("sheet1", "B1")]
    );
}
