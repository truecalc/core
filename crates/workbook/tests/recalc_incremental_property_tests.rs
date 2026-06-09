//! The issue's headline acceptance criterion: `incremental(edits) ≡ full()`.
//!
//! A random formula grid is built, fully recalculated, then mutated by a random
//! edit script. After each edit we recompute two ways — a fresh full recalc on
//! a clone, and an incremental recalc from the edited cell on the live workbook
//! — and assert the two grids are byte-identical (canonical JSON). Volatile
//! cells are exercised separately (they are always-dirty by design).

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

/// Build a workbook of `n` cells in column A: A1 a literal, each later cell a
/// formula over one or two earlier cells (so the graph is acyclic by
/// construction) seeded from `shape`.
fn build_grid(n: usize, shape: &[(u8, usize, usize)]) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr(1, 1), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    for i in 1..n {
        let row = (i + 1) as u32;
        let (op, p, q) = shape[i % shape.len()];
        // Reference strictly-earlier rows (1..=i) to stay acyclic.
        let p = (p % i) + 1; // 1..=i
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

    #[test]
    fn incremental_equals_full_after_random_edits(
        n in 3usize..12,
        shape in proptest::collection::vec((0u8..6, 0usize..20, 0usize..20), 4..10),
        edits in proptest::collection::vec((0usize..12, -5i64..5), 1..6),
    ) {
        let mut live = build_grid(n, &shape);
        live.recalc(&ctx());

        for (cell_i, new_val) in edits {
            // Edit a literal at the chosen row (overwriting whatever was there
            // with a literal keeps the graph valid and may change values).
            let row = ((cell_i % n) + 1) as u32;
            let a = addr(row, 1);
            // Only edit if it does not turn a formula cell into a literal in a
            // way that breaks downstream refs — a literal at any A-row is always
            // a valid input, so just overwrite.
            live.set("S", a, CellInput::Literal(Value::Number(new_val as f64))).unwrap();

            // Full recalc on a clone vs incremental on the live workbook.
            let mut full = live.clone();
            full.recalc(&ctx());
            live.recalc_incremental(&ctx(), &[("S".to_string(), a)]);

            prop_assert_eq!(
                live.to_json().unwrap(),
                full.to_json().unwrap(),
                "incremental and full grids diverged after editing {}",
                a.to_a1()
            );
        }
    }
}

/// A spill-aware grid on one sheet:
///   A1: an array anchor whose width we vary (spills across row 1),
///   B3: `=B1+1`   (reader of a spilled cell — column B is in A1's path),
///   C3: `=C1+1`   (reader of a cell A1 spills onto only when wide enough),
///   D3: `=SUM(B1:C1)` (range reader over spilled cells),
///   plus a *blocker* cell whose presence in row 1 blocks the spill.
/// Random edits then resize the array, and write/clear the blocker, so the
/// spill shrinks, grows, blocks, and unblocks. After each edit incremental
/// recalc must stay byte-identical to a full recalc (issue #591).
fn build_spill_grid() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr(1, 1), CellInput::Formula("={10,20}".into()))
        .unwrap(); // A1 spills A1:B1
    wb.set("S", addr(3, 2), CellInput::Formula("=B1+1".into()))
        .unwrap(); // B3 reads spilled B1
    wb.set("S", addr(3, 3), CellInput::Formula("=C1+1".into()))
        .unwrap(); // C3 reads C1 (spilled only when A1 is >= 3 wide)
    wb.set("S", addr(3, 4), CellInput::Formula("=SUM(B1:C1)".into()))
        .unwrap(); // D3 range-reads spilled cells
    wb
}

/// Array literals of widths 1..=4, used to drive shrink/grow. Width 1 collapses
/// to a scalar (no spill); 2..=4 spill across row 1.
fn array_formula(width: usize) -> CellInput {
    let elems: Vec<String> = (1..=width).map(|i| (i * 10).to_string()).collect();
    if width == 1 {
        CellInput::Formula(format!("={}", elems[0]))
    } else {
        CellInput::Formula(format!("={{{}}}", elems.join(",")))
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// `incremental ≡ full` across spill footprint and blocked-status changes:
    /// each edit resizes the anchor array (shrink/grow, incl. collapse to a
    /// scalar) or writes/clears a blocker in the spill path (block/unblock).
    #[test]
    fn incremental_equals_full_across_spill_transitions(
        // Each step: (anchor width 1..=4, blocker present, blocker col 2..=4).
        steps in proptest::collection::vec((1usize..=4, any::<bool>(), 2u32..=4), 1..8),
    ) {
        let mut live = build_spill_grid();
        live.recalc(&ctx());

        for (width, blocker, bcol) in steps {
            // Resize / collapse the anchor.
            live.set("S", addr(1, 1), array_formula(width)).unwrap();
            let mut edited = vec![("S".to_string(), addr(1, 1))];

            // Toggle a blocker in row 1 (a literal an array would spill onto).
            let blocker_addr = addr(1, bcol);
            if blocker {
                live.set("S", blocker_addr, CellInput::Literal(Value::Number(99.0)))
                    .unwrap();
            } else {
                live.clear("S", blocker_addr);
            }
            edited.push(("S".to_string(), blocker_addr));

            let mut full = live.clone();
            full.recalc(&ctx());
            live.recalc_incremental(&ctx(), &edited);

            prop_assert_eq!(
                live.to_json().unwrap(),
                full.to_json().unwrap(),
                "incremental and full diverged: width={} blocker={} bcol={}",
                width, blocker, bcol
            );
        }
    }
}

#[test]
fn incremental_recomputes_only_the_dirty_closure() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr(1, 1), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", addr(2, 1), CellInput::Formula("=A1+1".into()))
        .unwrap();
    wb.set("S", addr(3, 1), CellInput::Formula("=A2+1".into()))
        .unwrap();
    // Independent cell, not downstream of A1.
    wb.set("S", addr(1, 2), CellInput::Literal(Value::Number(100.0)))
        .unwrap();
    wb.set("S", addr(2, 2), CellInput::Formula("=B1+1".into()))
        .unwrap();
    wb.recalc(&ctx());

    // Edit A1; B-column must not appear in the change list.
    wb.set("S", addr(1, 1), CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    let changes = wb.recalc_incremental(&ctx(), &[("S".to_string(), addr(1, 1))]);
    let touched: Vec<String> = changes.iter().map(|c| c.addr.to_a1()).collect();
    assert!(touched.contains(&"A2".to_string()));
    assert!(touched.contains(&"A3".to_string()));
    assert!(
        !touched.iter().any(|t| t.starts_with('B')),
        "B column should be untouched: {touched:?}"
    );
}

#[test]
fn volatile_cells_are_always_dirty_in_incremental_recalc() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr(1, 1), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", addr(2, 1), CellInput::Formula("=TODAY()".into()))
        .unwrap();
    wb.recalc(&RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).unwrap()); // 2026-06-08

    // Edit an unrelated cell, but recalc with a *later* context: the volatile
    // TODAY() cell must still update even though it does not depend on A1.
    wb.set("S", addr(1, 1), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    let next = RecalcContext::new(1_780_878_600_000 + 86_400_000, "Etc/GMT", 0).unwrap();
    let changes = wb.recalc_incremental(&next, &[("S".to_string(), addr(1, 1))]);
    let touched: Vec<String> = changes.iter().map(|c| c.addr.to_a1()).collect();
    assert!(
        touched.contains(&"A2".to_string()),
        "volatile cell must be dirty: {touched:?}"
    );
    assert_eq!(
        wb.get("S", addr(2, 1)).unwrap().value(),
        &Value::Date(46182.0)
    );
}
