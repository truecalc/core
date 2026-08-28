//! `SheetIndex` answers every sheet lookup exactly as the linear scan it
//! replaced did (issue #952).
//!
//! Resolving a sheet name used to be
//! `sheets().iter().position(|s| simple_fold(s.name()) == target)` — a linear
//! walk that case-folded, and so allocated, **every sheet name it passed**.
//! Six places on the recalc path ran it once per formula cell, and graph build
//! ran it once per cross-sheet reference, which made recalculation
//! `O(cells × sheets × sheet-name-length)`. `SheetIndex` folds each sheet name
//! once per recalc and answers every lookup from a hash map.
//!
//! Case folding is **semantic** here — sheet names match case-insensitively
//! (schema spec §2) — so replacing the scan is a correctness change as much as
//! a performance one. [`Workbook::sheet_index`] is the same scan, still present
//! and untouched, and is used here as a test oracle: the index must agree with
//! it on every name, including names differing only by case, Unicode names
//! where simple folding is not a byte-wise lowercase, names that fold to the
//! same key, and a workbook at the maximum sheet count.
//!
//! The cost side of the same change — how many folds a recalc performs, and
//! whether that depends on the cell count or the tab-name length — is pinned by
//! `sheet_lookup_fold_count_tests.rs`, which needs a test binary to itself.

use truecalc_workbook::{
    Address, Cell, CellInput, CellRef, EngineFlavor, RecalcContext, SheetIndex, Value, Workbook,
    Worksheet,
};

/// `limits::MAX_SHEETS`, restated so the test fails loudly if the cap moves.
const MAX_SHEETS: usize = 256;

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

/// The folded form of `name`, through the crate's own folding (simple
/// `Case_Folding`, schema spec §2) rather than a reimplementation.
fn folded(name: &str) -> String {
    CellRef::from_display_name(name, Address::new(1, 1).unwrap()).sheet
}

/// The scan `SheetIndex` replaced, kept as the oracle every lookup is checked
/// against: first match wins, folding every name it passes.
fn scan_for_folded(wb: &Workbook, target_folded: &str) -> Option<usize> {
    wb.sheets()
        .iter()
        .position(|s| folded(s.name()) == target_folded)
}

/// Names chosen so that folding is not a byte-wise lowercase: Turkish dotted
/// and dotless `i` (`İ` U+0130 does **not** simple-fold to `i`, and `I` folds
/// to `i` while `ı` folds to itself), German `ß` (simple folding leaves it
/// alone — it does *not* become `ss`), and Greek sigma (`Σ` folds to `σ`, but
/// final `ς` folds to itself, so `ΟΔΟΣ` and `οδός` are **different** sheets
/// under simple folding even though full folding would unify them).
fn unicode_names() -> Vec<&'static str> {
    vec![
        "Data",
        "İstanbul",
        "Istanbul",
        "ıstanbul",
        "Straße",
        "STRASSE",
        "ΟΔΟΣ",
        "οδός",
        "Σigma",
    ]
}

#[test]
fn index_agrees_with_the_scan_on_unicode_and_case_variant_names() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for name in unicode_names() {
        wb.add_sheet(Worksheet::new(name)).unwrap();
    }
    let index = SheetIndex::build(&wb);

    // Every spelling anyone could reach these sheets by: the authored name,
    // its upper- and lower-cased forms, and a name that is not there at all.
    let mut queries: Vec<String> = Vec::new();
    for name in unicode_names() {
        queries.push(name.to_owned());
        queries.push(name.to_uppercase());
        queries.push(name.to_lowercase());
    }
    queries.push("nope".to_owned());
    queries.push("DATA ".to_owned()); // trailing space: a different sheet

    for q in &queries {
        assert_eq!(
            index.index_of_name(q),
            wb.sheet_index(q),
            "index_of_name({q:?}) disagreed with the linear scan"
        );
        assert_eq!(
            index.index_of_folded(&folded(q)),
            scan_for_folded(&wb, &folded(q)),
            "index_of_folded({:?}) disagreed with the linear scan",
            folded(q)
        );
    }

    // The Greek pair is the case that would silently pass under a wrong
    // (full-folding, or `to_lowercase`) implementation: these are two distinct
    // sheets, and each must resolve to itself.
    assert_ne!(folded("ΟΔΟΣ"), folded("οδός"));
    assert_ne!(index.index_of_name("ΟΔΟΣ"), index.index_of_name("οδός"));

    // Every sheet's folded name round-trips to its own tab.
    for (i, sheet) in wb.sheets().iter().enumerate() {
        assert_eq!(index.index_of_folded(&folded(sheet.name())), Some(i));
        assert_eq!(index.folded_name(i), folded(sheet.name()));
    }
}

#[test]
fn names_that_fold_to_the_same_key_resolve_first_wins_like_the_scan() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Data")).unwrap();
    wb.add_sheet(Worksheet::new("Other")).unwrap();
    // `add_sheet` rejects a folding collision, so the only way to build one is
    // the raw sheet-list accessor — which is public, so the lookup has to
    // answer for it.
    wb.sheets_mut().push(Worksheet::new("DATA"));

    let index = SheetIndex::build(&wb);
    assert_eq!(wb.sheet_index("DATA"), Some(0), "oracle is first-wins");
    for q in ["Data", "DATA", "data", "dAtA"] {
        assert_eq!(
            index.index_of_name(q),
            wb.sheet_index(q),
            "index_of_name({q:?}) must answer what the scan answers — the \
             *first* tab whose folded name matches, not the last"
        );
        assert_eq!(index.index_of_name(q), Some(0));
        assert_eq!(index.index_of_folded(&folded(q)), Some(0));
    }
    assert_eq!(index.index_of_name("Other"), Some(1));
    assert_eq!(index.len(), 3);
    assert!(!index.is_empty());
}

#[test]
fn index_agrees_with_the_scan_at_the_maximum_sheet_count() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for i in 0..MAX_SHEETS {
        wb.add_sheet(Worksheet::new(format!("Sheet {i}"))).unwrap();
    }
    assert_eq!(wb.sheets().len(), MAX_SHEETS);
    assert!(
        wb.add_sheet(Worksheet::new("one too many")).is_err(),
        "MAX_SHEETS is no longer {MAX_SHEETS}; this test is measuring the wrong cap"
    );

    let index = SheetIndex::build(&wb);
    for i in 0..MAX_SHEETS {
        let name = format!("Sheet {i}");
        let shouted = name.to_uppercase();
        assert_eq!(index.index_of_name(&name), Some(i));
        assert_eq!(index.index_of_name(&shouted), wb.sheet_index(&shouted));
        assert_eq!(index.index_of_name(&shouted), Some(i));
        assert_eq!(index.index_of_folded(&folded(&shouted)), Some(i));
        assert_eq!(index.folded_of_name(&shouted), Some(folded(&name).as_str()));
    }
    assert_eq!(index.index_of_name("Sheet 256"), None);
    assert_eq!(index.folded_of_name("Sheet 256"), None);
}

#[test]
fn cross_sheet_formulas_still_resolve_through_case_and_unicode() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Data")).unwrap();
    wb.add_sheet(Worksheet::new("Straße")).unwrap();
    wb.add_sheet(Worksheet::new("Report")).unwrap();

    let a1 = Address::new(1, 1).unwrap();
    wb.set("Data", a1, CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    wb.set("Straße", a1, CellInput::Literal(Value::Number(7.0)))
        .unwrap();
    // Both references spell the tab in a *different* case from the tab itself,
    // which is exactly what the folding in the lookup is for.
    wb.set(
        "Report",
        a1,
        CellInput::Formula("=DATA!A1 + straße!A1".to_owned()),
    )
    .unwrap();
    wb.set(
        "Report",
        Address::new(2, 1).unwrap(),
        CellInput::Formula("=SUM(data!A1:A1)".to_owned()),
    )
    .unwrap();

    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Report", a1).unwrap().value(),
        &Value::Number(17.0),
        "cross-sheet reads must resolve through the folded index"
    );
    assert_eq!(
        wb.get("Report", Address::new(2, 1).unwrap())
            .unwrap()
            .value(),
        &Value::Number(10.0)
    );

    // And an incremental recalc reaches the same answer.
    wb.set("Data", a1, CellInput::Literal(Value::Number(20.0)))
        .unwrap();
    let changes = wb.recalc_incremental(&ctx(), &[("Data".to_owned(), a1)]);
    assert!(!changes.is_empty());
    assert_eq!(wb.get("Report", a1).unwrap().value(), &Value::Number(27.0));
}

#[test]
fn many_sheet_workbook_recalculates_correctly() {
    // The shape the fold budget above is measured on, checked for its values:
    // an index that resolved the wrong tab would still be fast.
    let sheets = 64;
    let rows = 8;
    let mut wb = multi_sheet(sheets, rows, 3);
    wb.recalc(&ctx());
    for s in 0..sheets {
        let name = wb.sheets()[s].name().to_owned();
        for row in 1..=rows {
            assert_eq!(
                wb.get(&name, Address::new(row, 2).unwrap())
                    .unwrap()
                    .value(),
                &Value::Number(f64::from(row) + 1.0),
                "{name}!B{row}"
            );
        }
    }
}
