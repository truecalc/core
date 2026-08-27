//! Coverage for the dependency-graph queries the JavaScript workbook surface
//! exposes: `precedentsOf` and `dependentsOf`.
//!
//! These call the `truecalc_wasm_workbook::depgraph` functions the
//! `JsWorkbook` methods are thin wrappers over, against a
//! `truecalc_workbook::Workbook` — the same path the bindings take — so they
//! run natively under `cargo test` without a wasm runtime, matching this
//! crate's existing convention (see `round_trip.rs`, `table_bindings.rs`).

use truecalc_wasm_workbook::depgraph::{dependents_of, precedents_of, NameTargetRef, PrecedentRef};
use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn workbook(sheets: &[&str]) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for s in sheets {
        wb.add_sheet(Worksheet::new((*s).to_string())).unwrap();
    }
    wb
}

fn set_formula(wb: &mut Workbook, sheet: &str, a1: &str, formula: &str) {
    let addr = Address::from_a1(a1).unwrap();
    wb.set(sheet, addr, CellInput::Formula(formula.to_string()))
        .unwrap();
}

fn set_number(wb: &mut Workbook, sheet: &str, a1: &str, n: f64) {
    let addr = Address::from_a1(a1).unwrap();
    wb.set(sheet, addr, CellInput::Literal(Value::Number(n)))
        .unwrap();
}

/// `(depth, "Sheet!A1")` for each precedent that is a plain cell reference.
fn precedent_cells(wb: &Workbook, sheet: &str, a1: &str, depth: u32) -> Vec<(u32, String)> {
    precedents_of(wb, sheet, a1, Some(depth), None)
        .unwrap()
        .precedents
        .iter()
        .filter_map(|p| match &p.reference {
            PrecedentRef::Cell { sheet, a1 } => Some((p.depth, format!("{sheet}!{a1}"))),
            _ => None,
        })
        .collect()
}

fn dependent_cells(wb: &Workbook, sheet: &str, a1: &str, depth: u32) -> Vec<(u32, String)> {
    dependents_of(wb, sheet, a1, Some(depth), None)
        .unwrap()
        .dependents
        .iter()
        .map(|d| (d.depth, format!("{}!{}", d.sheet, d.a1)))
        .collect()
}

// --- A simple chain -------------------------------------------------------

/// A1 (literal) ← A2 ← A3: precedents walk up the chain, dependents walk down,
/// and each hop lands at the depth it actually sits at.
#[test]
fn chain_precedents_and_dependents_by_depth() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 10.0);
    set_formula(&mut wb, "Sheet1", "A2", "=A1+1");
    set_formula(&mut wb, "Sheet1", "A3", "=A2*2");

    assert_eq!(
        precedent_cells(&wb, "Sheet1", "A3", 1),
        vec![(1, "Sheet1!A2".to_string())]
    );
    assert_eq!(
        precedent_cells(&wb, "Sheet1", "A3", 5),
        vec![(1, "Sheet1!A2".to_string()), (2, "Sheet1!A1".to_string())]
    );

    assert_eq!(
        dependent_cells(&wb, "Sheet1", "A1", 1),
        vec![(1, "Sheet1!A2".to_string())]
    );
    assert_eq!(
        dependent_cells(&wb, "Sheet1", "A1", 5),
        vec![(1, "Sheet1!A2".to_string()), (2, "Sheet1!A3".to_string())]
    );
}

/// The whole chain fits inside the bounds, so nothing claims truncation.
#[test]
fn complete_walk_is_not_marked_truncated() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 10.0);
    set_formula(&mut wb, "Sheet1", "A2", "=A1+1");
    set_formula(&mut wb, "Sheet1", "A3", "=A2*2");

    let p = precedents_of(&wb, "Sheet1", "A3", Some(64), None).unwrap();
    assert!(!p.truncated);
    assert_eq!(p.truncated_by, None);

    let d = dependents_of(&wb, "Sheet1", "A1", Some(64), None).unwrap();
    assert!(!d.truncated);
    assert_eq!(d.truncated_by, None);
}

// --- Cross-sheet references ----------------------------------------------

/// A precedent on another sheet keeps its own sheet, in the workbook's casing,
/// and the reverse edge crosses back the other way.
#[test]
fn cross_sheet_reference_keeps_its_sheet() {
    let mut wb = workbook(&["Inputs", "Report"]);
    set_number(&mut wb, "Inputs", "B2", 7.0);
    set_formula(&mut wb, "Report", "A1", "=Inputs!B2*2");

    let p = precedents_of(&wb, "Report", "A1", None, None).unwrap();
    assert_eq!(p.cell.sheet, "Report");
    assert_eq!(p.cell.a1, "A1");
    assert_eq!(
        p.precedents
            .iter()
            .map(|n| n.reference.clone())
            .collect::<Vec<_>>(),
        vec![PrecedentRef::Cell {
            sheet: "Inputs".to_string(),
            a1: "B2".to_string(),
        }]
    );

    let d = dependents_of(&wb, "Inputs", "B2", None, None).unwrap();
    assert_eq!(d.dependents.len(), 1);
    assert_eq!(d.dependents[0].sheet, "Report");
    assert_eq!(d.dependents[0].a1, "A1");
}

/// A transitive walk crosses sheets repeatedly without flattening anything
/// onto the queried cell's sheet.
#[test]
fn transitive_walk_crosses_sheets() {
    let mut wb = workbook(&["A", "B", "C"]);
    set_number(&mut wb, "A", "A1", 1.0);
    set_formula(&mut wb, "B", "A1", "=A!A1+1");
    set_formula(&mut wb, "C", "A1", "=B!A1+1");

    assert_eq!(
        precedent_cells(&wb, "C", "A1", 3),
        vec![(1, "B!A1".to_string()), (2, "A!A1".to_string())]
    );
    assert_eq!(
        dependent_cells(&wb, "A", "A1", 3),
        vec![(1, "B!A1".to_string()), (2, "C!A1".to_string())]
    );
}

/// Sheet names are matched case-insensitively, and the answer always reports
/// the workbook's own casing rather than the caller's spelling.
#[test]
fn sheet_name_lookup_is_case_insensitive_and_answers_in_workbook_casing() {
    let mut wb = workbook(&["MySheet"]);
    set_number(&mut wb, "MySheet", "A1", 1.0);
    set_formula(&mut wb, "MySheet", "A2", "=A1");

    let p = precedents_of(&wb, "mYsHeEt", "A2", None, None).unwrap();
    assert_eq!(p.cell.sheet, "MySheet");
    assert_eq!(
        p.precedents[0].reference,
        PrecedentRef::Cell {
            sheet: "MySheet".to_string(),
            a1: "A1".to_string(),
        }
    );
}

// --- A cell with no precedents -------------------------------------------

/// A literal, an empty cell and a constant formula all answer with an empty
/// array — never a missing field, never an error.
#[test]
fn cells_without_precedents_return_an_empty_list() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 42.0);
    set_formula(&mut wb, "Sheet1", "A2", "=1+1");

    for a1 in ["A1", "A2", "Z99"] {
        let p = precedents_of(&wb, "Sheet1", a1, Some(64), None).unwrap();
        assert!(
            p.precedents.is_empty(),
            "{a1} should have no precedents, got {:?}",
            p.precedents
        );
        assert!(!p.truncated, "{a1} should not be marked truncated");
        assert_eq!(p.truncated_by, None);
    }

    // ... and the same on the dependents side for a cell nothing reads.
    let d = dependents_of(&wb, "Sheet1", "Z99", Some(64), None).unwrap();
    assert!(d.dependents.is_empty());
    assert!(!d.truncated);
}

/// The serialized form carries the empty list as a present, empty array, so a
/// JavaScript consumer never has to distinguish "no precedents" from "field
/// missing".
#[test]
fn empty_precedents_serialize_as_a_present_empty_array() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 42.0);

    let json =
        serde_json::to_value(precedents_of(&wb, "Sheet1", "A1", None, None).unwrap()).unwrap();
    assert_eq!(json["precedents"], serde_json::json!([]));
    assert_eq!(json["truncated"], serde_json::json!(false));
    assert!(
        json.get("truncatedBy").is_none(),
        "an untruncated result carries no reason: {json}"
    );
    assert_eq!(
        json["cell"],
        serde_json::json!({"sheet": "Sheet1", "a1": "A1"})
    );
}

// --- Ranges, names, unresolved refs --------------------------------------

/// A range is one precedent node, not one per member cell — the property that
/// keeps a `SUM(A1:A100000)` answer bounded.
#[test]
fn a_range_precedent_is_a_single_node() {
    let mut wb = workbook(&["Sheet1"]);
    set_formula(&mut wb, "Sheet1", "D1", "=SUM(A1:A1000)");

    let p = precedents_of(&wb, "Sheet1", "D1", Some(64), None).unwrap();
    assert_eq!(
        p.precedents
            .iter()
            .map(|n| n.reference.clone())
            .collect::<Vec<_>>(),
        vec![PrecedentRef::Range {
            sheet: "Sheet1".to_string(),
            range: "A1:A1000".to_string(),
        }]
    );
    assert!(!p.truncated);
}

/// A cell inside a range that a formula reads is a dependent of that formula,
/// without the caller knowing ranges exist.
#[test]
fn a_cell_inside_a_read_range_has_the_reader_as_a_dependent() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A5", 1.0);
    set_formula(&mut wb, "Sheet1", "D1", "=SUM(A1:A1000)");

    assert_eq!(
        dependent_cells(&wb, "Sheet1", "A5", 1),
        vec![(1, "Sheet1!D1".to_string())]
    );
}

/// A named-range precedent reports the name *and* what it currently points at,
/// so the indirection is never opaque to the caller.
#[test]
fn a_named_range_precedent_carries_its_target() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "B1", 3.0);
    wb.define_name("Rate", "Sheet1!B1").unwrap();
    set_formula(&mut wb, "Sheet1", "A1", "=Rate*2");

    let p = precedents_of(&wb, "Sheet1", "A1", None, None).unwrap();
    assert_eq!(
        p.precedents
            .iter()
            .map(|n| n.reference.clone())
            .collect::<Vec<_>>(),
        vec![PrecedentRef::Name {
            name: "Rate".to_string(),
            target: NameTargetRef::Cell {
                sheet: "Sheet1".to_string(),
                a1: "B1".to_string(),
            },
        }]
    );

    // The reverse edge follows the name through to its target cell.
    assert_eq!(
        dependent_cells(&wb, "Sheet1", "B1", 1),
        vec![(1, "Sheet1!A1".to_string())]
    );
}

/// A transitive walk follows a name through to its target's own precedents.
#[test]
fn transitive_walk_follows_a_name_to_its_targets_precedents() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "C1", 5.0);
    set_formula(&mut wb, "Sheet1", "B1", "=C1*10");
    wb.define_name("Rate", "Sheet1!B1").unwrap();
    set_formula(&mut wb, "Sheet1", "A1", "=Rate*2");

    assert_eq!(
        precedent_cells(&wb, "Sheet1", "A1", 2),
        vec![(2, "Sheet1!C1".to_string())]
    );
}

/// A reference to a sheet that does not exist is reported, not dropped: it is
/// why the cell will evaluate to an error.
#[test]
fn an_unresolvable_reference_is_reported() {
    let mut wb = workbook(&["Sheet1"]);
    set_formula(&mut wb, "Sheet1", "A1", "=Nope!B2");

    let p = precedents_of(&wb, "Sheet1", "A1", None, None).unwrap();
    assert_eq!(p.precedents.len(), 1);
    assert!(
        matches!(&p.precedents[0].reference, PrecedentRef::Unresolved { .. }),
        "expected an unresolved precedent, got {:?}",
        p.precedents[0].reference
    );
    assert!(!p.truncated);
}

// --- Cycles ---------------------------------------------------------------

/// A two-cell circular reference: the precedent/dependent walks over it
/// terminate at the bound instead of spinning on the cycle.
#[test]
fn a_cycle_terminates_precedent_and_dependent_walks() {
    let mut wb = workbook(&["Sheet1"]);
    set_formula(&mut wb, "Sheet1", "A1", "=B1+1");
    set_formula(&mut wb, "Sheet1", "B1", "=A1+1");

    assert_eq!(
        precedent_cells(&wb, "Sheet1", "A1", 64),
        vec![(1, "Sheet1!B1".to_string()), (2, "Sheet1!A1".to_string())]
    );
    assert_eq!(
        dependent_cells(&wb, "Sheet1", "A1", 64),
        vec![(1, "Sheet1!B1".to_string()), (2, "Sheet1!A1".to_string())]
    );
}

// --- The bounds being hit -------------------------------------------------

/// Stopping at `maxDepth` with more graph left is reported, so a shallow
/// answer is never mistaken for a complete one.
#[test]
fn hitting_max_depth_is_reported() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 1.0);
    set_formula(&mut wb, "Sheet1", "A2", "=A1+1");
    set_formula(&mut wb, "Sheet1", "A3", "=A2+1");
    set_formula(&mut wb, "Sheet1", "A4", "=A3+1");

    let p = precedents_of(&wb, "Sheet1", "A4", Some(2), None).unwrap();
    assert_eq!(p.precedents.len(), 2);
    assert!(p.truncated);
    assert_eq!(p.truncated_by.as_deref(), Some("maxDepth"));

    let d = dependents_of(&wb, "Sheet1", "A1", Some(1), None).unwrap();
    assert_eq!(d.dependents.len(), 1);
    assert!(d.truncated);
    assert_eq!(d.truncated_by.as_deref(), Some("maxDepth"));

    // The same walk with room to finish is not marked truncated.
    let full = precedents_of(&wb, "Sheet1", "A4", Some(3), None).unwrap();
    assert_eq!(full.precedents.len(), 3);
    assert!(!full.truncated);
}

/// Stopping at `maxNodes` is reported, and the payload really is capped.
#[test]
fn hitting_max_nodes_is_reported() {
    let mut wb = workbook(&["Sheet1"]);
    for row in 1..=20 {
        set_number(&mut wb, "Sheet1", &format!("A{row}"), row as f64);
    }
    set_formula(
        &mut wb,
        "Sheet1",
        "D1",
        "=A1+A2+A3+A4+A5+A6+A7+A8+A9+A10+A11+A12+A13+A14+A15+A16+A17+A18+A19+A20",
    );

    let p = precedents_of(&wb, "Sheet1", "D1", Some(64), Some(5)).unwrap();
    assert_eq!(p.precedents.len(), 5);
    assert!(p.truncated);
    assert_eq!(p.truncated_by.as_deref(), Some("maxNodes"));

    let full = precedents_of(&wb, "Sheet1", "D1", Some(64), Some(20)).unwrap();
    assert_eq!(full.precedents.len(), 20);
    assert!(!full.truncated);
}

/// `maxNodes` wins the reason when both bounds are reached: it is the bound
/// that actually ended the walk.
#[test]
fn max_nodes_is_the_reported_reason_when_both_bounds_bite() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 1.0);
    set_formula(&mut wb, "Sheet1", "A2", "=A1+1");
    set_formula(&mut wb, "Sheet1", "A3", "=A2+1");
    set_formula(&mut wb, "Sheet1", "A4", "=A3+1");

    let p = precedents_of(&wb, "Sheet1", "A4", Some(1), Some(0)).unwrap();
    assert!(p.precedents.is_empty());
    assert!(p.truncated);
    assert_eq!(p.truncated_by.as_deref(), Some("maxNodes"));
}

/// A request above the hard ceilings is clamped, and the clamp is only safe
/// because it still reports truncation.
#[test]
fn bounds_are_clamped_to_the_hard_ceilings() {
    let mut wb = workbook(&["Sheet1"]);
    // A chain 70 links long — deeper than MAX_MAX_DEPTH (64).
    set_number(&mut wb, "Sheet1", "A1", 1.0);
    for row in 2..=70 {
        set_formula(
            &mut wb,
            "Sheet1",
            &format!("A{row}"),
            &format!("=A{}+1", row - 1),
        );
    }

    let p = precedents_of(&wb, "Sheet1", "A70", Some(u32::MAX), None).unwrap();
    assert_eq!(p.precedents.len(), 64);
    assert!(p.truncated);
    assert_eq!(p.truncated_by.as_deref(), Some("maxDepth"));
}

/// A `maxDepth` of 0 is raised to 1 rather than returning an empty,
/// un-truncated answer that would read as "this cell reads nothing".
#[test]
fn zero_max_depth_is_raised_to_one() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 1.0);
    set_formula(&mut wb, "Sheet1", "A2", "=A1+1");

    let p = precedents_of(&wb, "Sheet1", "A2", Some(0), None).unwrap();
    assert_eq!(p.precedents.len(), 1);
    assert!(!p.truncated);
}

// --- Freshness after mutation --------------------------------------------

/// The graph is rebuilt per query, so an edit is visible immediately — with no
/// recalc in between, and in both directions.
#[test]
fn queries_see_edits_without_a_recalc() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 1.0);
    set_number(&mut wb, "Sheet1", "B1", 2.0);
    set_formula(&mut wb, "Sheet1", "C1", "=A1+1");

    assert_eq!(
        precedent_cells(&wb, "Sheet1", "C1", 1),
        vec![(1, "Sheet1!A1".to_string())]
    );

    // Repoint C1 at B1. No recalc.
    set_formula(&mut wb, "Sheet1", "C1", "=B1+1");
    assert_eq!(
        precedent_cells(&wb, "Sheet1", "C1", 1),
        vec![(1, "Sheet1!B1".to_string())]
    );
    assert!(dependent_cells(&wb, "Sheet1", "A1", 1).is_empty());
    assert_eq!(
        dependent_cells(&wb, "Sheet1", "B1", 1),
        vec![(1, "Sheet1!C1".to_string())]
    );

    // Clearing the formula removes the edge.
    wb.clear("Sheet1", Address::from_a1("C1").unwrap());
    assert!(dependent_cells(&wb, "Sheet1", "B1", 1).is_empty());
}

/// Retargeting a named range is visible immediately, without a recalc.
#[test]
fn retargeting_a_name_is_visible_immediately() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "B1", 1.0);
    set_number(&mut wb, "Sheet1", "B2", 2.0);
    wb.define_name("Rate", "Sheet1!B1").unwrap();
    set_formula(&mut wb, "Sheet1", "A1", "=Rate*2");

    assert_eq!(
        dependent_cells(&wb, "Sheet1", "B1", 1),
        vec![(1, "Sheet1!A1".to_string())]
    );

    wb.redefine_name("Rate", "Sheet1!B2").unwrap();
    assert!(dependent_cells(&wb, "Sheet1", "B1", 1).is_empty());
    assert_eq!(
        dependent_cells(&wb, "Sheet1", "B2", 1),
        vec![(1, "Sheet1!A1".to_string())]
    );
}

/// A recalc does not change the answers: the graph is a function of the
/// formulas, not of the values they produced.
#[test]
fn recalc_does_not_change_the_answers() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 1.0);
    set_formula(&mut wb, "Sheet1", "A2", "=A1+1");

    let before = precedents_of(&wb, "Sheet1", "A2", Some(64), None).unwrap();
    let ctx = RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).unwrap();
    wb.recalc(&ctx);
    let after = precedents_of(&wb, "Sheet1", "A2", Some(64), None).unwrap();
    assert_eq!(before, after);
}

// --- Bad arguments --------------------------------------------------------

/// An unknown sheet is an error, not an empty answer — a typo must not read as
/// "this cell depends on nothing".
#[test]
fn an_unknown_sheet_is_an_error() {
    let wb = workbook(&["Sheet1"]);
    assert!(precedents_of(&wb, "Nope", "A1", None, None).is_err());
    assert!(dependents_of(&wb, "Nope", "A1", None, None).is_err());
}

/// A malformed address is an error for the same reason.
#[test]
fn a_malformed_address_is_an_error() {
    let wb = workbook(&["Sheet1"]);
    assert!(precedents_of(&wb, "Sheet1", "not-an-address", None, None).is_err());
    assert!(dependents_of(&wb, "Sheet1", "A0", None, None).is_err());
}

/// A truncated result carries its reason as a present field.
#[test]
fn a_truncated_result_serializes_its_reason() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 1.0);
    set_formula(&mut wb, "Sheet1", "A2", "=A1+1");
    set_formula(&mut wb, "Sheet1", "A3", "=A2+1");

    let json =
        serde_json::to_value(precedents_of(&wb, "Sheet1", "A3", Some(1), None).unwrap()).unwrap();
    assert_eq!(json["truncated"], serde_json::json!(true));
    assert_eq!(json["truncatedBy"], serde_json::json!("maxDepth"));
}

/// A formula reading a name that is not defined reports the name with an
/// explicit unresolved target, never a missing field.
#[test]
fn an_undefined_name_reports_an_unresolved_target() {
    let mut wb = workbook(&["Sheet1"]);
    // The name is never defined, so the reference does not resolve at all and
    // the graph records it as unresolved rather than as a name node.
    set_formula(&mut wb, "Sheet1", "A1", "=NoSuchName*2");
    let p = precedents_of(&wb, "Sheet1", "A1", None, None).unwrap();
    assert_eq!(p.precedents.len(), 1);
    assert!(matches!(
        &p.precedents[0].reference,
        PrecedentRef::Unresolved { .. }
    ));
}

/// A depth-bounded walk that has nothing further to report is **not** marked
/// truncated. The bound being reached is not the same as the answer being
/// incomplete, and a `maxDepth: 1` query that cried wolf on every formula
/// would make `truncated` worthless.
#[test]
fn reaching_the_depth_bound_with_nothing_left_is_not_truncation() {
    let mut wb = workbook(&["Sheet1"]);
    set_number(&mut wb, "Sheet1", "A1", 1.0);
    set_formula(&mut wb, "Sheet1", "A2", "=A1+1");

    // A2's only precedent is a literal — depth 2 would add nothing.
    let p = precedents_of(&wb, "Sheet1", "A2", Some(1), None).unwrap();
    assert_eq!(p.precedents.len(), 1);
    assert!(!p.truncated, "nothing left to walk, so not truncated");
    assert_eq!(p.truncated_by, None);

    // A1's only dependent is A2, which nothing reads — depth 2 would add
    // nothing either.
    let d = dependents_of(&wb, "Sheet1", "A1", Some(1), None).unwrap();
    assert_eq!(d.dependents.len(), 1);
    assert!(!d.truncated);
    assert_eq!(d.truncated_by, None);

    // But add a reader of A2 and the same depth-1 query is now genuinely a
    // prefix, and says so.
    set_formula(&mut wb, "Sheet1", "A3", "=A2+1");
    let d = dependents_of(&wb, "Sheet1", "A1", Some(1), None).unwrap();
    assert!(d.truncated);
    assert_eq!(d.truncated_by.as_deref(), Some("maxDepth"));
}

/// A cycle already fully reported is not re-reported as truncation, however
/// shallow the bound.
#[test]
fn a_fully_reported_cycle_is_not_truncation() {
    let mut wb = workbook(&["Sheet1"]);
    set_formula(&mut wb, "Sheet1", "A1", "=B1+1");
    set_formula(&mut wb, "Sheet1", "B1", "=A1+1");

    let d = dependents_of(&wb, "Sheet1", "A1", Some(2), None).unwrap();
    assert_eq!(d.dependents.len(), 2);
    assert!(!d.truncated);
}
