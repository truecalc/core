//! Coverage for issue #977: `JsWorkbook.removeSheet`/`.renameSheet`/
//! `.moveSheet`/`.insertSheet`, `.totalCells`, `.get`, the new `anchor` field
//! on `.resolved`, and `.recalcIncremental` all wrap
//! `truecalc_workbook::Workbook`'s equivalents 1:1, the same pattern
//! `defineName`/`redefineName`/`defineTable`/`redefineTable` already use.
//!
//! These exercise `truecalc_workbook::Workbook` directly — the same path
//! `JsWorkbook`'s wrappers call through to — so they run natively under
//! `cargo test` without a wasm runtime, matching this crate's existing
//! convention (see round_trip.rs, table_bindings.rs).

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Resolved, Value, Workbook, Worksheet,
};

fn sheets_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1".to_string())).unwrap();
    wb
}

fn set_formula(wb: &mut Workbook, sheet: &str, a1: &str, formula: &str) -> Result<(), String> {
    let addr = Address::from_a1(a1).unwrap();
    wb.set(sheet, addr, CellInput::Formula(formula.to_string()))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn set_literal(wb: &mut Workbook, sheet: &str, a1: &str, value: Value) {
    let addr = Address::from_a1(a1).unwrap();
    wb.set(sheet, addr, CellInput::Literal(value)).unwrap();
}

fn recalc(wb: &mut Workbook) {
    let ctx = RecalcContext::new(0, "UTC", 0).unwrap();
    wb.recalc(&ctx);
}

fn resolved(wb: &Workbook, sheet: &str, a1: &str) -> Option<Resolved> {
    let addr = Address::from_a1(a1).unwrap();
    wb.resolved(sheet, addr)
}

// ---- sheet management (item 1) --------------------------------------------

/// `removeSheet` wraps `Workbook::remove_sheet`: an unknown name is a silent
/// no-op (matching `removeName`), and removing an existing sheet drops it
/// from `sheets()`.
#[test]
fn remove_sheet_drops_it_and_is_a_noop_on_unknown_name() {
    let mut wb = sheets_workbook();
    wb.add_sheet(Worksheet::new("Sheet2".to_string())).unwrap();
    assert_eq!(wb.sheets().len(), 2);

    wb.remove_sheet("Nope"); // silent no-op
    assert_eq!(wb.sheets().len(), 2);

    wb.remove_sheet("sheet1"); // case-insensitive
    assert_eq!(wb.sheets().len(), 1);
    assert!(wb.sheet("Sheet1").is_none());
    assert!(wb.sheet("Sheet2").is_some());
}

/// `renameSheet` wraps `Workbook::rename_sheet`, including its holistic
/// repoint of formula references to the renamed sheet.
#[test]
fn rename_sheet_repoints_cross_sheet_formula() {
    let mut wb = sheets_workbook();
    wb.add_sheet(Worksheet::new("Sheet2".to_string())).unwrap();
    set_formula(&mut wb, "Sheet1", "A1", "=10").unwrap();
    set_formula(&mut wb, "Sheet2", "A1", "=Sheet1!A1+1").unwrap();

    wb.rename_sheet("Sheet1", "Renamed").unwrap();
    assert!(wb.sheet("Sheet1").is_none());
    assert!(wb.sheet("Renamed").is_some());

    recalc(&mut wb);
    assert_eq!(
        resolved(&wb, "Sheet2", "A1").map(|r| r.value),
        Some(Value::Number(11.0))
    );
}

/// `renameSheet` surfaces `Workbook::rename_sheet`'s error on an unknown
/// source sheet, leaving the workbook untouched.
#[test]
fn rename_sheet_errors_on_unknown_source() {
    let mut wb = sheets_workbook();
    assert!(wb.rename_sheet("Ghost", "New").is_err());
    assert!(wb.sheet("Ghost").is_none());
    assert!(wb.sheet("New").is_none());
}

/// `moveSheet` wraps `Workbook::move_sheet`'s 0-based tab-position reorder.
#[test]
fn move_sheet_reorders_tabs() {
    let mut wb = sheets_workbook();
    wb.add_sheet(Worksheet::new("Sheet2".to_string())).unwrap();
    wb.add_sheet(Worksheet::new("Sheet3".to_string())).unwrap();
    assert_eq!(
        wb.sheets().iter().map(|s| s.name()).collect::<Vec<_>>(),
        vec!["Sheet1", "Sheet2", "Sheet3"]
    );

    wb.move_sheet(0, 2).unwrap();
    assert_eq!(
        wb.sheets().iter().map(|s| s.name()).collect::<Vec<_>>(),
        vec!["Sheet2", "Sheet3", "Sheet1"]
    );
}

/// `moveSheet` surfaces `Workbook::move_sheet`'s out-of-range error.
#[test]
fn move_sheet_errors_out_of_range() {
    let mut wb = sheets_workbook();
    assert!(wb.move_sheet(0, 5).is_err());
}

/// `insertSheet` wraps `Workbook::insert_sheet`, constructing the
/// `Worksheet` exactly as `addSheet` does. Inserting mid-workbook shifts
/// later tabs right.
#[test]
fn insert_sheet_at_position_shifts_later_tabs() {
    let mut wb = sheets_workbook();
    wb.add_sheet(Worksheet::new("Sheet2".to_string())).unwrap();

    wb.insert_sheet(1, Worksheet::new("Middle".to_string()))
        .unwrap();
    assert_eq!(
        wb.sheets().iter().map(|s| s.name()).collect::<Vec<_>>(),
        vec!["Sheet1", "Middle", "Sheet2"]
    );
}

/// `insertSheet` at `index == len` appends, the same as `addSheet`.
#[test]
fn insert_sheet_at_len_appends() {
    let mut wb = sheets_workbook();
    let len = wb.sheets().len();
    wb.insert_sheet(len, Worksheet::new("Appended".to_string()))
        .unwrap();
    assert_eq!(wb.sheets().last().unwrap().name(), "Appended");
}

// ---- totalCells (item 2) ---------------------------------------------------

/// `totalCells` wraps `Workbook::total_cells`, counting populated cells
/// across every sheet.
#[test]
fn total_cells_counts_across_sheets() {
    let mut wb = sheets_workbook();
    wb.add_sheet(Worksheet::new("Sheet2".to_string())).unwrap();
    set_literal(&mut wb, "Sheet1", "A1", Value::Number(1.0));
    set_literal(&mut wb, "Sheet2", "A1", Value::Number(2.0));
    set_literal(&mut wb, "Sheet2", "A2", Value::Number(3.0));
    assert_eq!(wb.total_cells(), 3);
}

// ---- get (item 3) — authored cell only -------------------------------------

/// `get` wraps `Workbook::get`: it returns only the **authored** cell, not
/// the resolved value — a spilled non-anchor cell is not authored and so is
/// absent, distinguishing it from `resolved`.
#[test]
fn get_returns_none_for_absent_cell_but_resolved_finds_a_spill_element() {
    let mut wb = sheets_workbook();
    set_formula(&mut wb, "Sheet1", "A1", "={1,2;3,4}").unwrap();
    recalc(&mut wb);

    // B1 (row 1, col 2) is inside the spill but not authored.
    let addr = Address::from_a1("B1").unwrap();
    assert!(
        wb.get("Sheet1", addr).is_none(),
        "a spilled non-anchor cell is not authored"
    );
    // `resolved` reconstructs its value and reports the spill anchor.
    let r = resolved(&wb, "Sheet1", "B1").expect("resolved should find the spilled element");
    assert_eq!(r.value, Value::Number(2.0));
    assert_eq!(r.anchor, Address::from_a1("A1"));
}

/// `get` returns the authored cell's formula text and last-evaluated value.
#[test]
fn get_returns_authored_formula_and_value() {
    let mut wb = sheets_workbook();
    set_formula(&mut wb, "Sheet1", "A1", "=1+1").unwrap();
    recalc(&mut wb);

    let addr = Address::from_a1("A1").unwrap();
    let cell = wb.get("Sheet1", addr).expect("A1 is authored");
    assert_eq!(cell.formula(), Some("=1+1"));
    assert_eq!(cell.value(), &Value::Number(2.0));
}

// ---- resolved's additive `anchor` (item 4) ---------------------------------

/// An authored cell's `Resolved` carries no anchor — the additive-only
/// contract the `anchor` field on the wasm binding's JSON depends on.
#[test]
fn resolved_anchor_is_none_for_an_authored_cell() {
    let mut wb = sheets_workbook();
    set_literal(&mut wb, "Sheet1", "A1", Value::Number(1.0));
    let r = resolved(&wb, "Sheet1", "A1").unwrap();
    assert_eq!(r.anchor, None);
}

/// A spilled cell's `Resolved` carries the anchor address, which the wasm
/// binding surfaces as the new `anchor` JSON key.
#[test]
fn resolved_anchor_is_set_for_a_spilled_cell() {
    let mut wb = sheets_workbook();
    set_formula(&mut wb, "Sheet1", "A1", "={1,2}").unwrap();
    recalc(&mut wb);
    let r = resolved(&wb, "Sheet1", "B1").unwrap();
    assert_eq!(r.anchor, Address::from_a1("A1"));
}

// ---- recalcIncremental (item 5) --------------------------------------------

/// `recalcIncremental` wraps `Workbook::recalc_incremental`: after a full
/// recalc, editing one cell and running the incremental path recomputes only
/// its dependents and returns their changes.
#[test]
fn recalc_incremental_recomputes_only_dependents_of_the_edit() {
    let mut wb = sheets_workbook();
    set_formula(&mut wb, "Sheet1", "A1", "=10").unwrap();
    set_formula(&mut wb, "Sheet1", "B1", "=A1+1").unwrap();
    recalc(&mut wb);
    assert_eq!(
        resolved(&wb, "Sheet1", "B1").map(|r| r.value),
        Some(Value::Number(11.0))
    );

    // Edit A1 directly (bypassing `set`'s own recalc trigger) then drive the
    // incremental path exactly as the binding would.
    let addr_a1 = Address::from_a1("A1").unwrap();
    wb.set("Sheet1", addr_a1, CellInput::Formula("=20".into()))
        .unwrap();

    let ctx = RecalcContext::new(0, "UTC", 0).unwrap();
    let edited = vec![("Sheet1".to_string(), addr_a1)];
    let changes = wb.recalc_incremental(&ctx, &edited);

    // Both A1 (the edit itself) and B1 (its dependent) recompute.
    assert!(changes
        .iter()
        .any(|c| c.sheet == "Sheet1" && c.addr.to_a1() == "A1"));
    assert!(changes
        .iter()
        .any(|c| c.sheet == "Sheet1" && c.addr.to_a1() == "B1"));
    assert_eq!(
        resolved(&wb, "Sheet1", "B1").map(|r| r.value),
        Some(Value::Number(21.0))
    );
}

/// An empty `edited` list is valid input for `recalcIncremental`, not an
/// error — matching `Workbook::recalc_incremental`'s own semantics.
#[test]
fn recalc_incremental_accepts_empty_edited_list() {
    let mut wb = sheets_workbook();
    set_formula(&mut wb, "Sheet1", "A1", "=1").unwrap();
    recalc(&mut wb);

    let ctx = RecalcContext::new(0, "UTC", 0).unwrap();
    let changes = wb.recalc_incremental(&ctx, &[]);
    assert!(changes.is_empty());
}
