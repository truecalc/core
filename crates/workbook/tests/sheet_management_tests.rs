//! Sheet management: add / insert / remove / rename / move with position
//! semantics, plus name uniqueness under Unicode *simple* case folding
//! (plan item 3.1, schema spec §2–§3, scope ADR Decision 5).

use truecalc_workbook::{Cell, EngineFlavor, Value, Workbook, Worksheet};

fn wb() -> Workbook {
    Workbook::new(EngineFlavor::Sheets)
}

fn names(wb: &Workbook) -> Vec<&str> {
    wb.sheets().iter().map(Worksheet::name).collect()
}

#[test]
fn add_sheet_appends_in_tab_order() {
    let mut wb = wb();
    assert_eq!(wb.add_sheet(Worksheet::new("Alpha")).unwrap(), 0);
    assert_eq!(wb.add_sheet(Worksheet::new("Beta")).unwrap(), 1);
    assert_eq!(names(&wb), vec!["Alpha", "Beta"]);
}

#[test]
fn insert_sheet_shifts_later_tabs_right() {
    let mut wb = wb();
    wb.add_sheet(Worksheet::new("Alpha")).unwrap();
    wb.add_sheet(Worksheet::new("Gamma")).unwrap();
    wb.insert_sheet(1, Worksheet::new("Beta")).unwrap();
    assert_eq!(names(&wb), vec!["Alpha", "Beta", "Gamma"]);
}

#[test]
fn insert_at_len_appends_and_beyond_len_errors() {
    let mut wb = wb();
    wb.add_sheet(Worksheet::new("Alpha")).unwrap();
    wb.insert_sheet(1, Worksheet::new("Beta")).unwrap(); // == len: appends
    assert_eq!(names(&wb), vec!["Alpha", "Beta"]);
    assert!(wb.insert_sheet(99, Worksheet::new("Far")).is_err());
}

#[test]
fn lookup_is_case_insensitive() {
    let mut wb = wb();
    wb.add_sheet(Worksheet::new("Budget")).unwrap();
    assert!(wb.sheet("budget").is_some());
    assert!(wb.sheet("BUDGET").is_some());
    assert_eq!(wb.sheet_index("BuDgEt"), Some(0));
    assert!(wb.sheet("Other").is_none());
}

#[test]
fn duplicate_name_is_rejected_case_insensitively() {
    let mut wb = wb();
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    assert!(wb.add_sheet(Worksheet::new("sheet1")).is_err());
    assert!(wb.add_sheet(Worksheet::new("SHEET1")).is_err());
    assert_eq!(
        names(&wb),
        vec!["Sheet1"],
        "rejected adds leave the workbook unchanged"
    );
}

#[test]
fn uniqueness_uses_simple_case_folding_not_ascii_lowercasing() {
    // Kelvin sign (U+212A) simple-folds to ASCII 'k'; a sheet named "K" must
    // collide with one named "\u{212A}" (KELVIN SIGN). ASCII-only lowercasing
    // would miss this — the test pins the simple-case-folding rule (§2).
    let mut wb = wb();
    wb.add_sheet(Worksheet::new("K")).unwrap();
    assert!(
        wb.add_sheet(Worksheet::new("\u{212A}")).is_err(),
        "Kelvin sign must collide with ASCII K under simple case folding"
    );
}

#[test]
fn remove_sheet_shifts_later_tabs_left() {
    let mut wb = wb();
    for n in ["Alpha", "Beta", "Gamma"] {
        wb.add_sheet(Worksheet::new(n)).unwrap();
    }
    let removed = wb.remove_sheet("beta").unwrap();
    assert_eq!(removed.name(), "Beta");
    assert_eq!(names(&wb), vec!["Alpha", "Gamma"]);
    assert!(wb.remove_sheet("missing").is_none());
}

#[test]
fn rename_keeps_cells_and_position() {
    let mut wb = wb();
    let mut ws = Worksheet::new("Old");
    ws.set(
        truecalc_workbook::Address::from_a1("A1").unwrap(),
        Cell::literal(Value::Number(5.0)).unwrap(),
    );
    wb.add_sheet(ws).unwrap();
    wb.add_sheet(Worksheet::new("Other")).unwrap();
    wb.rename_sheet("old", "New").unwrap();
    assert_eq!(names(&wb), vec!["New", "Other"]);
    assert_eq!(wb.sheet("New").unwrap().len(), 1, "cells survive a rename");
}

#[test]
fn rename_to_a_pure_case_change_of_itself_is_allowed() {
    let mut wb = wb();
    wb.add_sheet(Worksheet::new("data")).unwrap();
    wb.rename_sheet("data", "DATA").unwrap();
    assert_eq!(names(&wb), vec!["DATA"]);
}

#[test]
fn rename_to_a_different_sheets_name_is_rejected() {
    let mut wb = wb();
    wb.add_sheet(Worksheet::new("Alpha")).unwrap();
    wb.add_sheet(Worksheet::new("Beta")).unwrap();
    assert!(wb.rename_sheet("Alpha", "beta").is_err());
    assert!(wb.rename_sheet("Missing", "X").is_err());
    assert_eq!(names(&wb), vec!["Alpha", "Beta"]);
}

#[test]
fn move_sheet_repositions_with_shift() {
    let mut wb = wb();
    for n in ["Alpha", "Beta", "Gamma"] {
        wb.add_sheet(Worksheet::new(n)).unwrap();
    }
    wb.move_sheet(0, 2).unwrap(); // Alpha to the end
    assert_eq!(names(&wb), vec!["Beta", "Gamma", "Alpha"]);
    wb.move_sheet(2, 0).unwrap(); // back to the front
    assert_eq!(names(&wb), vec!["Alpha", "Beta", "Gamma"]);
    assert!(wb.move_sheet(0, 99).is_err());
}

#[test]
fn empty_and_too_long_names_are_rejected() {
    let mut wb = wb();
    assert!(wb.add_sheet(Worksheet::new("")).is_err());
    let too_long: String = "x".repeat(101);
    assert!(wb.add_sheet(Worksheet::new(too_long)).is_err());
    let max_len: String = "x".repeat(100);
    assert!(wb.add_sheet(Worksheet::new(max_len)).is_ok());
}

#[test]
fn sheet_mut_edits_the_grid_in_place() {
    let mut wb = wb();
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb.sheet_mut("sheet1").unwrap().set(
        truecalc_workbook::Address::from_a1("A1").unwrap(),
        Cell::literal(Value::Number(9.0)).unwrap(),
    );
    assert_eq!(wb.sheet("Sheet1").unwrap().len(), 1);
}
