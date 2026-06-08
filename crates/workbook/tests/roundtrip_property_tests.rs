//! Property tests (issue #531): canonical round-trip byte identity
//! (`to_json ∘ from_json = id`), hash stability across round-trips, and a
//! generator of structurally valid workbooks. Schema-validation of the golden
//! files lives in `schema_validation_tests.rs`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use proptest::prelude::*;
use truecalc_workbook::{Cell, EngineFlavor, NamedRange, Value, Workbook, Worksheet};

fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

/// A finite f64 that is representable in canonical form (no NaN/Inf).
fn finite_f64() -> impl Strategy<Value = f64> {
    prop_oneof![
        any::<f64>().prop_filter("finite", |x| x.is_finite()),
        Just(0.0),
        Just(-0.0),
        Just(1e21),
        Just(1e-7),
        Just(1e-6),
        Just(0.1 + 0.2),
        (-1000i64..1000).prop_map(|n| n as f64),
    ]
}

/// A scalar (non-array) value.
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
            Just("#NUM!"),
        ]
        .prop_map(|c| Value::Error(c.to_owned())),
        Just(Value::Empty),
        finite_f64().prop_map(Value::Date),
    ]
}

/// A scalar value that is valid as an *anchor array element* (no Empty padding
/// restriction here — empty elements are allowed in arrays per the spec).
fn array_element() -> impl Strategy<Value = Value> {
    scalar_value()
}

/// A 2-D array value (≥ 2 cells, rectangular) for spill anchors.
fn array_value() -> impl Strategy<Value = Value> {
    (1usize..3, 1usize..3)
        .prop_filter("not 1x1", |(r, c)| !(*r == 1 && *c == 1))
        .prop_flat_map(|(rows, cols)| {
            proptest::collection::vec(
                proptest::collection::vec(array_element(), cols..=cols),
                rows..=rows,
            )
        })
        .prop_map(Value::Array)
}

/// A literal cell value (never Empty — empty literal is rejected) or a formula
/// cell. Returns the Cell plus the spill dims if it is an array anchor.
fn cell_strategy() -> impl Strategy<Value = Cell> {
    prop_oneof![
        // Literal scalar (non-empty).
        scalar_value()
            .prop_filter("non-empty literal", |v| !matches!(v, Value::Empty))
            .prop_map(|v| Cell::literal(v).unwrap()),
        // Formula cell with a scalar value (incl. empty before recalc).
        ("=[A-Z0-9+*/() ]{0,16}", scalar_value()).prop_map(|(f, v)| Cell::with_formula(f, v)),
    ]
}

/// A small, well-spaced set of A1 addresses that never collide and leave room
/// so a 2x2 array anchor never overlaps a neighbour.
fn spaced_addresses(n: usize) -> Vec<String> {
    // Columns A, D, G, ...; rows 1, 4, 7, ... — 3 apart in both axes.
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

/// A worksheet with up to a few well-spaced cells; at most one array anchor to
/// keep spill rectangles trivially non-overlapping.
fn worksheet_strategy(name: String) -> impl Strategy<Value = Worksheet> {
    (0usize..4)
        .prop_flat_map(|n| {
            (
                proptest::collection::vec(cell_strategy(), n..=n),
                proptest::option::of(array_value()),
            )
        })
        .prop_map(move |(cells, anchor)| {
            let mut ws = Worksheet::new(name.clone());
            let addrs = spaced_addresses(cells.len() + usize::from(anchor.is_some()));
            let mut it = addrs.into_iter();
            for cell in cells {
                if let Some(addr) = it.next() {
                    ws.cells_mut().insert(addr, cell);
                }
            }
            if let Some(arr) = anchor {
                if let Some(addr) = it.next() {
                    ws.cells_mut()
                        .insert(addr, Cell::with_formula("=SEQUENCE(2,2)", arr));
                }
            }
            ws
        })
}

/// A workbook with unique sheet names (Sheet0, Sheet1, …) and named ranges
/// that target existing sheets with canonical refs.
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
            // Up to 3 named ranges, each targeting an existing sheet; none when
            // there are no sheets (a ref must target a real sheet).
            let max_names = if n_sheets == 0 { 0usize } else { 3 };
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// to_json ∘ from_json = id, byte-for-byte (schema spec §8 guarantee).
    #[test]
    fn round_trip_is_byte_identical(wb in workbook_strategy()) {
        let json = wb.to_json().expect("serialize");
        let back = Workbook::from_json(json.as_bytes()).expect("deserialize");
        let json2 = back.to_json().expect("re-serialize");
        prop_assert_eq!(json, json2);
    }

    /// from_json output re-serializes identically and is structurally equal.
    #[test]
    fn parsed_workbook_equals_original(wb in workbook_strategy()) {
        let json = wb.to_json().expect("serialize");
        let back = Workbook::from_json(json.as_bytes()).expect("deserialize");
        prop_assert_eq!(&wb, &back);
    }

    /// Hash is stable across a canonical round-trip (schema spec §8: hash and
    /// float equality operate on post-normalization bit patterns).
    #[test]
    fn hash_is_stable_across_round_trip(wb in workbook_strategy()) {
        let json = wb.to_json().expect("serialize");
        let back = Workbook::from_json(json.as_bytes()).expect("deserialize");
        prop_assert_eq!(hash_of(&wb), hash_of(&back));
    }

    /// Canonical output is idempotent: parsing canonical bytes and
    /// re-serializing yields the same bytes (and a second parse too).
    #[test]
    fn canonicalization_is_idempotent(wb in workbook_strategy()) {
        let once = wb.to_json().expect("serialize");
        let parsed = Workbook::from_json(once.as_bytes()).expect("parse");
        let twice = parsed.to_json().expect("re-serialize");
        prop_assert_eq!(&once, &twice);
        let parsed2 = Workbook::from_json(twice.as_bytes()).expect("re-parse");
        prop_assert_eq!(twice, parsed2.to_json().expect("third serialize"));
    }
}
