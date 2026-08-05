//! Property tests for clone independence (P4.1 / #538).
//!
//! The property: given any generated workbook, clone it, apply a mutation to
//! the clone (set a cell, run recalc), and assert the original workbook is
//! unchanged (same value from `get()`, same recalc result). Also assert the
//! clone has the expected mutation.

use proptest::prelude::*;
use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).unwrap()
}

fn addr(row: u32, col: u32) -> Address {
    Address::new(row, col).unwrap()
}

/// Build a simple formula grid of `n` cells in column A (same pattern as
/// `recalc_incremental_property_tests.rs`).
fn build_grid(n: usize, shape: &[(u8, usize, usize)]) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr(1, 1), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    for i in 1..n {
        let row = (i + 1) as u32;
        let (op, p, q) = shape[i % shape.len()];
        let p = (p % i) + 1;
        let q = (q % i) + 1;
        let formula = match op % 3 {
            0 => format!("=A{}+A{}", p, q),
            1 => format!("=A{}*2", p),
            _ => format!("=SUM(A{}:A{})", p.min(q), p.max(q)),
        };
        wb.set("S", addr(row, 1), CellInput::Formula(formula))
            .unwrap();
    }
    wb
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Mutating a clone does not affect the original's cell values.
    ///
    /// Build a grid, fully recalc, snapshot the original's canonical JSON,
    /// clone it, mutate the clone (overwrite one cell with a new literal),
    /// recalc the clone, and assert the original JSON is unchanged.
    #[test]
    fn clone_mutation_does_not_affect_original(
        n in 3usize..12,
        shape in proptest::collection::vec((0u8..6, 0usize..20, 0usize..20), 4..10),
        // (cell_index, new_value) pairs
        mutations in proptest::collection::vec((0usize..12, -5i64..5), 1..5),
    ) {
        let mut original = build_grid(n, &shape);
        original.recalc(&ctx());

        // Snapshot the original before any cloning/mutation.
        let original_json_before = original.to_json().unwrap();

        // Clone and mutate.
        let mut clone = original.clone();
        for (cell_i, new_val) in &mutations {
            let row = ((cell_i % n) + 1) as u32;
            let a = addr(row, 1);
            clone.set("S", a, CellInput::Literal(Value::Number(*new_val as f64))).unwrap();
        }
        clone.recalc(&ctx());

        // Original must be identical to its pre-clone snapshot.
        let original_json_after = original.to_json().unwrap();
        prop_assert_eq!(
            &original_json_before,
            &original_json_after,
            "original workbook changed after mutating clone"
        );
    }

    /// The clone reflects the expected mutation: after overwriting one cell in
    /// the clone with a known literal and doing a full recalc, that cell in the
    /// clone holds the written value while the same cell in the original is
    /// unchanged.
    #[test]
    fn clone_reflects_mutation_and_original_is_unchanged(
        n in 3usize..12,
        shape in proptest::collection::vec((0u8..6, 0usize..20, 0usize..20), 4..10),
        target_i in 0usize..12,
        new_val in -100i64..100,
    ) {
        let mut original = build_grid(n, &shape);
        original.recalc(&ctx());

        let target_row = ((target_i % n) + 1) as u32;
        let target_addr = addr(target_row, 1);

        // Remember what the original has at target_addr (may be None if no
        // cell is set; in the grid we always set rows 1..=n so it exists).
        let original_value_before = original
            .get("S", target_addr)
            .map(|c| c.value().clone());

        // Clone, overwrite, and recalc the clone.
        let mut clone = original.clone();
        let new_literal = Value::Number(new_val as f64);
        clone
            .set("S", target_addr, CellInput::Literal(new_literal.clone()))
            .unwrap();
        clone.recalc(&ctx());

        // The clone's target cell must be the written literal.
        let clone_value = clone.get("S", target_addr).map(|c| c.value().clone());
        prop_assert_eq!(
            clone_value,
            Some(new_literal.clone()),
            "clone did not reflect the written value at {}",
            target_addr.to_a1()
        );

        // The original's target cell must be unchanged.
        let original_value_after = original
            .get("S", target_addr)
            .map(|c| c.value().clone());
        prop_assert_eq!(
            original_value_before,
            original_value_after,
            "original cell {} changed after mutating clone",
            target_addr.to_a1()
        );
    }

    /// Recalculating the clone does not affect the original's recalc output.
    ///
    /// Build, recalc original, snapshot. Clone and recalc with an *identical*
    /// context (idempotent recalc). The original's JSON must remain the same.
    #[test]
    fn clone_recalc_does_not_affect_original(
        n in 3usize..12,
        shape in proptest::collection::vec((0u8..6, 0usize..20, 0usize..20), 4..10),
    ) {
        let mut original = build_grid(n, &shape);
        original.recalc(&ctx());

        let original_json_before = original.to_json().unwrap();

        let mut clone = original.clone();
        clone.recalc(&ctx());

        prop_assert_eq!(
            &original_json_before,
            &original.to_json().unwrap(),
            "original workbook changed after recalculating clone"
        );
    }
}

/// Deterministic (non-property) sanity check: a single edit to the clone must
/// not bleed back into the original, even across the dependency graph.
#[test]
fn clone_edit_does_not_bleed_into_original() {
    let mut original = Workbook::new(EngineFlavor::Sheets);
    original.add_sheet(Worksheet::new("S")).unwrap();
    // A1 = 1, A2 = A1+1, A3 = A2+1
    original
        .set("S", addr(1, 1), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    original
        .set("S", addr(2, 1), CellInput::Formula("=A1+1".into()))
        .unwrap();
    original
        .set("S", addr(3, 1), CellInput::Formula("=A2+1".into()))
        .unwrap();
    original.recalc(&ctx());

    let original_a1 = original.get("S", addr(1, 1)).unwrap().value().clone();
    let original_a3 = original.get("S", addr(3, 1)).unwrap().value().clone();

    // Clone and mutate root.
    let mut clone = original.clone();
    clone
        .set("S", addr(1, 1), CellInput::Literal(Value::Number(100.0)))
        .unwrap();
    clone.recalc(&ctx());

    // Clone has the new value.
    assert_eq!(clone.get("S", addr(1, 1)).unwrap().value(), &Value::Number(100.0));
    assert_eq!(clone.get("S", addr(3, 1)).unwrap().value(), &Value::Number(102.0));

    // Original is untouched.
    assert_eq!(original.get("S", addr(1, 1)).unwrap().value(), &original_a1);
    assert_eq!(original.get("S", addr(3, 1)).unwrap().value(), &original_a3);
}

/// Clearing a cell in the clone must not affect the original.
#[test]
fn clone_clear_does_not_affect_original() {
    let mut original = Workbook::new(EngineFlavor::Sheets);
    original.add_sheet(Worksheet::new("S")).unwrap();
    original
        .set("S", addr(1, 1), CellInput::Literal(Value::Number(42.0)))
        .unwrap();
    original.recalc(&ctx());

    let original_json = original.to_json().unwrap();

    let mut clone = original.clone();
    clone.clear("S", addr(1, 1));
    clone.recalc(&ctx());

    // Original unchanged.
    assert_eq!(original.to_json().unwrap(), original_json);
    // Clone cell is gone.
    assert!(clone.get("S", addr(1, 1)).is_none());
}
