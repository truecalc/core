//! How many case folds one recalculation performs, and what that number
//! depends on (issue #952).
//!
//! Resolving a cell's sheet used to be a linear `position` scan that
//! case-folded — and so allocated — every sheet name it passed, run once per
//! formula cell in six places on the recalc path. `SheetIndex` folds the sheet
//! list once per recalc and answers every lookup from a hash map.
//!
//! **Exact counts, not wall clock.** [`folds_performed`] reports how many case
//! folds this process has performed and over how many input bytes; a fold
//! allocates and walks its input, so those two numbers *are* the cost this
//! change is about, and unlike a stopwatch they are identical on every machine
//! and in either build profile. This is the same standard the allocation-count
//! tests in this crate already use.
//!
//! The headline is the **marginal** count: how many folds does one more formula
//! cell cost? Differencing two recalcs of the same shape cancels the fixed
//! per-recalc cost, and the answer must be exactly zero — a per-cell cost of
//! zero is what makes both the sheet count and the sheet-name length drop out
//! of the per-cell term entirely. Measured on this change:
//!
//! | | before | after |
//! |---|---:|---:|
//! | folds per extra formula cell, full recalc, 10 sheets | 16.50 | **0** |
//! | folds per extra formula cell, incremental, 10 sheets | 27.50 | **0** |
//! | folds per warm full recalc, 200 sheets × 10 rows | 603,600 | **600** |
//! | folds per incremental recalc, 200 sheets × 10 rows | 1,006,204 | **1,401** |
//!
//! **This file owns its test binary.** The counters are process-wide, so a
//! sibling `#[test]` running concurrently in the same binary would be counted
//! into the measurement — `grid_lookup_alloc_tests.rs` documents the same
//! constraint for its allocation counter and solves it by keeping everything in
//! one `#[test]`. Adding a second test to this file reintroduces the hazard.
//!
//! Correctness — that the index resolves the *same* sheet the scan did — is
//! pinned by `sheet_index_lookup_tests.rs`. An index that answered nothing
//! correctly would pass every assertion here.

use truecalc_workbook::{
    folds_performed, Address, Cell, CellInput, EngineFlavor, RecalcContext, Value, Workbook,
    Worksheet,
};

fn ctx() -> RecalcContext {
    RecalcContext::new(0, "UTC", 0).expect("UTC is valid")
}

/// `sheets` tabs, each `rows` rows of `A{r}` literal + `B{r} = =A{r}+1`, with
/// tab names padded to `name_len` characters.
///
/// Built through `Worksheet::cells_mut` rather than `Workbook::set` so the
/// fixture itself does not dominate the run: `set` resolves its sheet by name
/// too, and the shapes here are deliberately many-sheet.
fn multi_sheet(sheets: usize, rows: u32, name_len: usize) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for s in 0..sheets {
        let stem = format!("S{s}");
        let pad = "x".repeat(name_len.saturating_sub(stem.len()));
        let mut ws = Worksheet::new(format!("{stem}{pad}"));
        for row in 1..=rows {
            ws.cells_mut().insert(
                Address::new(row, 1).unwrap().to_a1(),
                Cell::literal(Value::Number(f64::from(row))).unwrap(),
            );
            ws.cells_mut().insert(
                Address::new(row, 2).unwrap().to_a1(),
                Cell::with_formula(format!("=A{row}+1"), Value::Empty),
            );
        }
        wb.add_sheet(ws).unwrap();
    }
    wb
}

/// Like [`multi_sheet`], but every formula is a **qualified cross-sheet**
/// reference into the first tab (`=S0!A{r}+1`) rather than a bare one.
///
/// A bare reference never names a sheet, so it exercises none of the sheet
/// lookup on the evaluation path; a qualified one resolves its target sheet on
/// every evaluation of every cell. Both shapes are measured, because the
/// evaluation-side lookup and the grid-side one are separate call sites.
fn multi_sheet_cross(sheets: usize, rows: u32, name_len: usize) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for s in 0..sheets {
        let stem = format!("S{s}");
        let pad = "x".repeat(name_len.saturating_sub(stem.len()));
        wb.add_sheet(Worksheet::new(format!("{stem}{pad}")))
            .unwrap();
    }
    let source = wb.sheets()[0].name().to_owned();
    for s in 0..sheets {
        for row in 1..=rows {
            wb.sheets_mut()[s].cells_mut().insert(
                Address::new(row, 1).unwrap().to_a1(),
                Cell::literal(Value::Number(f64::from(row))).unwrap(),
            );
            wb.sheets_mut()[s].cells_mut().insert(
                Address::new(row, 2).unwrap().to_a1(),
                Cell::with_formula(format!("={source}!A{row}+1"), Value::Empty),
            );
        }
    }
    wb
}

/// Folds performed, and bytes folded, while running `body`.
fn folds_during<T>(body: impl FnOnce() -> T) -> (u64, u64) {
    let (calls_before, bytes_before) = folds_performed();
    drop(body());
    let (calls_after, bytes_after) = folds_performed();
    (calls_after - calls_before, bytes_after - bytes_before)
}

/// Folds performed by one **warm** full recalc of the given shape (the graph
/// cache and the stored grid are both warmed first, so this is the steady-state
/// recalculation cost, not a first-load cost).
fn folds_to_recalc(sheets: usize, rows: u32, name_len: usize) -> (u64, u64) {
    let mut wb = multi_sheet(sheets, rows, name_len);
    wb.recalc(&ctx());
    folds_during(|| wb.recalc(&ctx()))
}

/// [`folds_to_recalc`] for the qualified cross-sheet shape.
fn folds_to_recalc_cross(sheets: usize, rows: u32, name_len: usize) -> (u64, u64) {
    let mut wb = multi_sheet_cross(sheets, rows, name_len);
    wb.recalc(&ctx());
    folds_during(|| wb.recalc(&ctx()))
}

/// Folds performed by editing one cell and recalculating incrementally, from a
/// warm workbook.
fn folds_to_edit_and_recalc(sheets: usize, rows: u32, name_len: usize) -> (u64, u64) {
    let mut wb = multi_sheet(sheets, rows, name_len);
    wb.recalc(&ctx());
    let sheet = wb.sheets()[0].name().to_owned();
    let a1 = Address::new(1, 1).unwrap();
    wb.set(&sheet, a1, CellInput::Literal(Value::Number(99.0)))
        .unwrap();
    folds_during(|| wb.recalc_incremental(&ctx(), &[(sheet.clone(), a1)]))
}

/// The one test in this binary — see the module docs for why it must stay the
/// only one.
#[test]
fn sheet_lookup_costs_no_folds_per_cell_and_none_per_name_character() {
    // Warm the lazily initialised machinery (function registry, ICU data) so
    // first-touch work is not attributed to a measured run.
    let _ = folds_to_recalc(2, 4, 3);

    // ── 1. A full recalc folds nothing per formula cell ──────────────────
    // Same sheet count, twice the rows: everything that scales with the sheet
    // list cancels, leaving only what each extra formula cell costs.
    let (small_calls, small_bytes) = folds_to_recalc(10, 10, 3);
    let (large_calls, large_bytes) = folds_to_recalc(10, 20, 3);
    let extra_cells = 10 * (20 - 10);
    assert_eq!(
        large_calls,
        small_calls,
        "a full recalc folded {} extra times for {extra_cells} extra formula cells \
         ({small_calls} folds at 100 cells, {large_calls} at 200); resolving a \
         cell's sheet must be a map probe against the per-recalc index, never a \
         scan that folds sheet names",
        large_calls as i64 - small_calls as i64
    );
    assert_eq!(
        large_bytes,
        small_bytes,
        "a full recalc folded {} extra bytes for {extra_cells} extra formula \
         cells; while any sheet-name bytes are folded per cell, tab-name length \
         is still a performance parameter",
        large_bytes as i64 - small_bytes as i64
    );

    // ── 2. Nor does an incremental recalc ────────────────────────────────
    // Its slope used to be the steeper of the two: it ran three further
    // per-formula-cell scans (the volatile sweep, the pre-edit snapshot, and
    // spill seeding) on top of evaluation's.
    let (small_calls, small_bytes) = folds_to_edit_and_recalc(10, 10, 3);
    let (large_calls, large_bytes) = folds_to_edit_and_recalc(10, 20, 3);
    assert_eq!(
        large_calls,
        small_calls,
        "an incremental recalc folded {} extra times for {extra_cells} extra \
         formula cells ({small_calls} folds at 100 cells, {large_calls} at 200)",
        large_calls as i64 - small_calls as i64
    );
    assert_eq!(
        large_bytes,
        small_bytes,
        "an incremental recalc folded {} extra bytes for {extra_cells} extra \
         formula cells",
        large_bytes as i64 - small_bytes as i64
    );

    // ── 3. Sheet-name length changes nothing about how many folds run ────
    // A 60-character tab name is 20× a 3-character one. Before this change the
    // fold *count* was unchanged by that too — but each of those folds walked
    // 20× the bytes, and there were `O(cells × sheets)` of them, which is why
    // renaming `S17` to `Cash Flow Statement 2027` cost 3.6× the wall clock.
    // Now the only folds left are the index build, so the byte total is
    // bounded by the sheet list itself and cannot scale with the grid.
    let (short_calls, short_bytes) = folds_to_recalc(10, 10, 3);
    let (long_calls, long_bytes) = folds_to_recalc(10, 10, 60);
    assert_eq!(
        long_calls, short_calls,
        "a recalc of the same grid folded {short_calls} times with 3-character \
         tab names and {long_calls} times with 60-character ones"
    );
    assert!(
        long_bytes <= 10 * 60 * 8,
        "a recalc of a 10-sheet, 100-formula-cell workbook with 60-character tab \
         names folded {long_bytes} bytes (with 3-character names: \
         {short_bytes}); that must stay bounded by the sheet list (10 names × \
         60 characters, a small constant number of times), not by the grid"
    );

    // ── 4. Nor does a *cross-sheet* formula, which names its sheet ───────
    // Every formula here is `=S0!A{r}+1`, so evaluating one resolves a sheet
    // name explicitly — a lookup a bare `=A{r}+1` never performs. It must cost
    // no fold either, and cost the same whether the tab is called `S0` or
    // carries a 60-character name.
    let (small_calls, small_bytes) = folds_to_recalc_cross(10, 10, 3);
    let (large_calls, large_bytes) = folds_to_recalc_cross(10, 20, 3);
    assert_eq!(
        large_calls,
        small_calls,
        "a full recalc of cross-sheet formulas folded {} extra times for \
         {extra_cells} extra formula cells ({small_calls} folds at 100 cells, \
         {large_calls} at 200); resolving a reference's target sheet must be a \
         map probe, not a scan",
        large_calls as i64 - small_calls as i64
    );
    assert_eq!(
        large_bytes,
        small_bytes,
        "a full recalc of cross-sheet formulas folded {} extra bytes for \
         {extra_cells} extra formula cells",
        large_bytes as i64 - small_bytes as i64
    );
    let (long_cross_calls, _) = folds_to_recalc_cross(10, 10, 60);
    assert_eq!(
        long_cross_calls, small_calls,
        "a recalc of the same cross-sheet grid folded {small_calls} times with \
         3-character tab names and {long_cross_calls} times with 60-character ones"
    );

    // ── 5. Total folds per recalc are O(sheets), not O(cells × sheets) ───
    // Measured before this change: 603,600 folds for the full recalc below and
    // 1,006,204 for the incremental one.
    let sheets = 200;
    let (full_calls, _) = folds_to_recalc(sheets, 10, 3);
    let (inc_calls, _) = folds_to_edit_and_recalc(sheets, 10, 3);
    let budget = 8 * sheets as u64 + 8;
    assert!(
        full_calls <= budget,
        "a warm full recalc of {sheets} sheets × 10 rows performed {full_calls} \
         folds (budget {budget}); it must fold the sheet list a constant number \
         of times, not once per formula cell"
    );
    assert!(
        inc_calls <= budget,
        "an incremental recalc of {sheets} sheets × 10 rows performed \
         {inc_calls} folds (budget {budget})"
    );
}
