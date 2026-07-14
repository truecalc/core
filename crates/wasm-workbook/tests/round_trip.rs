//! Round-trip tests for issue #715: a host calls `translateFormula(...)` and
//! then feeds the rewritten text straight back into the workbook via `set`.
//!
//! These exercise `truecalc_workbook::Workbook` directly — the same parse +
//! evaluate path `JsWorkbook.set` wraps — so they run natively under
//! `cargo nextest`/`cargo test` without a wasm runtime.

use truecalc_wasm_workbook::translate_formula;
use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet};

fn sheets_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1".to_string())).unwrap();
    wb
}

fn set_formula(wb: &mut Workbook, a1: &str, formula: &str) -> Result<(), String> {
    let addr = Address::from_a1(a1).unwrap();
    wb.set("Sheet1", addr, CellInput::Formula(formula.to_string()))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn recalc(wb: &mut Workbook) {
    let ctx = RecalcContext::new(0, "UTC", 0).unwrap();
    wb.recalc(&ctx);
}

fn resolved(wb: &Workbook, a1: &str) -> Option<Value> {
    let addr = Address::from_a1(a1).unwrap();
    wb.resolved("Sheet1", addr).map(|r| r.value)
}

/// 3.0's parser accepts every `$`-absolute shape on `set` — the shapes a
/// tokenizer emits after a fill/paste adjustment. (Before 3.0 these threw a
/// parse error at `set` time; issue #715 gap 2.)
#[test]
fn set_accepts_all_absolute_reference_shapes() {
    let mut wb = sheets_workbook();
    for formula in [
        "=$A$1",
        "=$A1",
        "=A$1",
        "=SUM($A$1:$A$2)",
        "=Sheet1!$A$1",
    ] {
        assert!(
            set_formula(&mut wb, "C1", formula).is_ok(),
            "set should accept absolute-reference formula {formula:?}"
        );
    }
}

/// `set` doesn't just parse `$`-absolute refs — it evaluates them. `=$A$1`
/// resolves to A1's value after recalc.
#[test]
fn absolute_reference_evaluates_after_recalc() {
    let mut wb = sheets_workbook();
    set_formula(&mut wb, "A1", "=41").unwrap(); // seed A1 = 41 via a formula
    set_formula(&mut wb, "B1", "=$A$1+1").unwrap();
    recalc(&mut wb);
    assert_eq!(resolved(&wb, "B1"), Some(Value::Number(42.0)));
}

/// The fill/paste loop end to end: translate a formula by an offset, then set
/// the rewritten text and confirm it evaluates. `=A1+$A$2` filled down one row
/// becomes `=A2+$A$2` (relative row shifts, absolute stays), which evaluates
/// against the shifted operands.
#[test]
fn translate_then_set_round_trips() {
    let mut wb = sheets_workbook();
    set_formula(&mut wb, "A1", "=10").unwrap();
    set_formula(&mut wb, "A2", "=20").unwrap();

    let translated = translate_formula("=A1+$A$2", 1, 0);
    let formula = translated.formula.expect("translation should succeed");
    assert_eq!(formula, "=A2+$A$2");

    // A2 (=20) + $A$2 (=20) = 40.
    set_formula(&mut wb, "B2", &formula).unwrap();
    recalc(&mut wb);
    assert_eq!(resolved(&wb, "B2"), Some(Value::Number(40.0)));
}

/// Documents issue #715 gap 3 (`#REF!` error literal): the parser does not yet
/// accept `#REF!` as an error literal, so `set` rejects a formula whose text an
/// out-of-bounds fill produced. Tracked as a follow-up; this test guards the
/// current behavior and should be updated (to `is_ok()` + a `#REF!` value
/// assertion) when the literal lands.
#[test]
fn set_still_rejects_ref_error_literal() {
    let mut wb = sheets_workbook();
    assert!(
        set_formula(&mut wb, "C1", "=#REF!").is_err(),
        "#REF! literal is not yet a parseable value (follow-up)"
    );
    assert!(
        set_formula(&mut wb, "C2", "=#REF!+1").is_err(),
        "#REF! literal is not yet a parseable value (follow-up)"
    );
}
