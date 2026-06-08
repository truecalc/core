//! Hash operates on post-normalization f64 bit patterns (scope ADR /
//! schema spec §8) and is consistent with `==`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use truecalc_workbook::{Cell, EngineFlavor, Value, Workbook, Worksheet};

fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

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
