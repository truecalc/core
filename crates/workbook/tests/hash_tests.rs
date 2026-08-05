//! Hash operates on post-normalization f64 bit patterns (scope ADR /
//! schema spec §8) and is consistent with `==`.
//!
//! Property tests (P4.1 / #538): equal workbooks must hash identically —
//! `wb_a == wb_b → hash(wb_a) == hash(wb_b)`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use proptest::prelude::*;
use truecalc_workbook::{Cell, CellInput, EngineFlavor, NamedRange, Value, Workbook, Worksheet};

fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

// ---------------------------------------------------------------------------
// Strategies (minimal subset from roundtrip_property_tests.rs)
// ---------------------------------------------------------------------------

fn finite_f64() -> impl Strategy<Value = f64> {
    prop_oneof![
        any::<f64>().prop_filter("finite", |x| x.is_finite()),
        Just(0.0_f64),
        Just(-0.0_f64),
        (-1000i64..1000).prop_map(|n| n as f64),
    ]
}

fn scalar_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        finite_f64().prop_map(Value::Number),
        ".*".prop_map(Value::Text),
        any::<bool>().prop_map(Value::Boolean),
        prop_oneof![
            Just("#REF!"),
            Just("#DIV/0!"),
            Just("#VALUE!"),
            Just("#N/A"),
        ]
        .prop_map(|c| Value::Error(c.to_owned())),
        Just(Value::Empty),
        finite_f64().prop_map(Value::Date),
    ]
}

fn cell_strategy() -> impl Strategy<Value = Cell> {
    prop_oneof![
        scalar_value()
            .prop_filter("non-empty literal", |v| !matches!(v, Value::Empty))
            .prop_map(|v| Cell::literal(v).unwrap()),
        ("=[A-Z0-9+*/() ]{0,16}", scalar_value())
            .prop_map(|(f, v)| Cell::with_formula(f, v)),
    ]
}

fn spaced_addresses(n: usize) -> Vec<String> {
    let mut out = Vec::new();
    let cols = ["A", "D", "G", "J"];
    let rows = [1u32, 4, 7, 10];
    'outer: for &r in &rows {
        for &c in &cols {
            out.push(format!("{c}{r}"));
            if out.len() == n {
                break 'outer;
            }
        }
    }
    out
}

fn worksheet_strategy(name: String) -> impl Strategy<Value = Worksheet> {
    (0usize..4)
        .prop_flat_map(|n| proptest::collection::vec(cell_strategy(), n..=n))
        .prop_map(move |cells| {
            let mut ws = Worksheet::new(name.clone());
            let addrs = spaced_addresses(cells.len());
            for (addr, cell) in addrs.into_iter().zip(cells) {
                ws.cells_mut().insert(addr, cell);
            }
            ws
        })
}

fn workbook_strategy() -> impl Strategy<Value = Workbook> {
    let engine = prop_oneof![Just(EngineFlavor::Sheets), Just(EngineFlavor::Excel)];
    (engine, 0usize..4)
        .prop_flat_map(|(engine, n_sheets)| {
            let sheets = (0..n_sheets)
                .map(|i| worksheet_strategy(format!("Sheet{i}")).boxed())
                .collect::<Vec<_>>();
            (Just(engine), Just(n_sheets), sheets)
        })
        .prop_flat_map(|(engine, n_sheets, sheets)| {
            let max_names = if n_sheets == 0 { 0usize } else { 2 };
            (
                Just(engine),
                Just(sheets),
                Just(n_sheets),
                proptest::collection::vec(0usize..n_sheets.max(1), 0..=max_names),
            )
        })
        .prop_map(|(engine, sheets, n_sheets, name_targets)| {
            let mut wb = Workbook::new(engine);
            for ws in sheets {
                wb.sheets_mut().push(ws);
            }
            if n_sheets > 0 {
                for (i, target) in name_targets.into_iter().enumerate() {
                    let sheet_idx = target % n_sheets;
                    wb.names_mut().push(NamedRange {
                        name: format!("Name{i}"),
                        r#ref: format!("Sheet{sheet_idx}!A1"),
                    });
                }
            }
            wb
        })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[test]
fn negative_zero_equals_and_hashes_like_zero() {
    assert_eq!(Value::Number(-0.0), Value::Number(0.0));
    assert_eq!(hash_of(&Value::Number(-0.0)), hash_of(&Value::Number(0.0)));
    assert_eq!(hash_of(&Value::Date(-0.0)), hash_of(&Value::Date(0.0)));
}

#[test]
fn number_and_date_with_same_serial_are_distinct() {
    assert_ne!(Value::Number(46180.0), Value::Date(46180.0));
    assert_ne!(
        hash_of(&Value::Number(46180.0)),
        hash_of(&Value::Date(46180.0))
    );
}

#[test]
fn equal_workbooks_hash_equal() {
    let build = || {
        let mut wb = Workbook::new(EngineFlavor::Sheets);
        let mut sheet = Worksheet::new("Sheet1");
        sheet
            .cells_mut()
            .insert("A1".to_owned(), Cell::literal(Value::Number(-0.0)).unwrap());
        wb.sheets_mut().push(sheet);
        wb
    };
    let a = build();
    let mut b = build();
    // Overwrite A1 with positive zero: still equal post-normalization.
    b.sheets_mut()[0]
        .cells_mut()
        .insert("A1".to_owned(), Cell::literal(Value::Number(0.0)).unwrap());
    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn different_engines_hash_differently() {
    let sheets = Workbook::new(EngineFlavor::Sheets);
    let excel = Workbook::new(EngineFlavor::Excel);
    assert_ne!(sheets, excel);
    assert_ne!(hash_of(&sheets), hash_of(&excel));
}

// ---------------------------------------------------------------------------
// Property tests (P4.1 / #538)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Core P4.1 property: equal workbooks must hash identically.
    /// A clone is structurally equal to its source; both must hash the same.
    #[test]
    fn equal_workbooks_hash_identically(wb in workbook_strategy()) {
        let clone = wb.clone();
        prop_assert_eq!(&wb, &clone, "clone must equal original");
        prop_assert_eq!(
            hash_of(&wb),
            hash_of(&clone),
            "equal workbooks must have the same hash"
        );
    }

    /// Mutation that restores original state must preserve equality and hash.
    /// We set a cell then clear it, returning the workbook to its prior state.
    #[test]
    fn set_then_clear_preserves_hash(wb in workbook_strategy()) {
        // Only meaningful when there is at least one sheet to mutate.
        prop_assume!(!wb.sheets().is_empty());

        let original = wb.clone();

        // Mutate: write a value into a cell that was previously absent (A1 on
        // the first sheet is guaranteed clear by our strategy's spaced_addresses
        // which starts at row 1 col A — but only inserts that cell if n>=1;
        // using a distinct address (Z9) that no strategy cell ever occupies).
        let mut mutated = wb.clone();
        let sheet_name = mutated.sheets()[0].name().to_owned();

        // Write then immediately clear — the workbook returns to its original state.
        use truecalc_workbook::Address;
        let addr = Address::from_a1("Z9").unwrap();
        mutated
            .set(&sheet_name, addr, CellInput::Literal(Value::Number(42.0)))
            .unwrap();
        mutated.clear(&sheet_name, addr);

        prop_assert_eq!(
            &original,
            &mutated,
            "set+clear must restore structural equality"
        );
        prop_assert_eq!(
            hash_of(&original),
            hash_of(&mutated),
            "set+clear must restore hash equality"
        );
    }
}
