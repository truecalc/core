//! `RecalcContext` and volatile pinning (P3.3 / scope ADR Decision 3): volatile
//! functions read only from the context, the UTC→local serial conversion uses
//! the vendored tz database, and same workbook + same context ⇒ identical grid.

use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook};

fn a1(s: &str) -> Address {
    Address::from_a1(s).unwrap()
}

fn wb_with(formula: &str) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(truecalc_workbook::Worksheet::new("Sheet1"))
        .unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Formula(formula.into()))
        .unwrap();
    wb
}

#[test]
fn now_serial_localizes_against_the_vendored_timezone() {
    // 2026-06-08T00:30:00Z. Under Etc/GMT local == UTC ⇒ day serial 46181,
    // time 00:30 ⇒ fraction 0.5/24.
    let ms = 1_780_878_600_000; // 2026-06-08T00:30:00Z
    let gmt = RecalcContext::new(ms, "Etc/GMT", 0).unwrap();
    let s = gmt.now_serial().unwrap();
    assert!((s - (46181.0 + 0.5 / 24.0)).abs() < 1e-9, "got {s}");

    // Same instant in America/New_York (UTC-4 in June, DST) is the previous
    // evening (2026-06-07 20:30 local) ⇒ day serial 46180.
    let ny = RecalcContext::new(ms, "America/New_York", 0).unwrap();
    let sn = ny.now_serial().unwrap();
    assert_eq!(
        sn.floor(),
        46180.0,
        "NY local date is the day before, got {sn}"
    );
}

#[test]
fn today_is_pinned_by_the_context_not_the_wall_clock() {
    // Etc/GMT, instant on 2026-06-08 ⇒ TODAY() == 46181 (date typed).
    let ctx = RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).unwrap();
    let mut wb = wb_with("=TODAY()");
    wb.recalc(&ctx);
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Date(46181.0)
    );
}

#[test]
fn now_minus_now_in_one_recalc_is_zero() {
    let ctx = RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).unwrap();
    let mut wb = wb_with("=TODAY()-TODAY()");
    wb.recalc(&ctx);
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Number(0.0)
    );
}

#[test]
fn same_context_is_byte_identical() {
    let ctx = RecalcContext::new(1_780_878_600_000, "Etc/GMT", 7).unwrap();
    let build = || {
        let mut wb = Workbook::new(EngineFlavor::Sheets);
        wb.add_sheet(truecalc_workbook::Worksheet::new("Sheet1"))
            .unwrap();
        wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(3.0)))
            .unwrap();
        wb.set("Sheet1", a1("A2"), CellInput::Formula("=A1*TODAY()".into()))
            .unwrap();
        wb.set("Sheet1", a1("A3"), CellInput::Formula("=A2+1".into()))
            .unwrap();
        wb
    };
    let mut w1 = build();
    let mut w2 = build();
    w1.recalc(&ctx);
    w2.recalc(&ctx);
    assert_eq!(
        w1.to_json().unwrap(),
        w2.to_json().unwrap(),
        "same workbook + same context ⇒ byte-identical canonical JSON"
    );
}

#[test]
fn different_contexts_legitimately_differ() {
    let mut early = wb_with("=TODAY()");
    let mut late = wb_with("=TODAY()");
    early.recalc(&RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).unwrap()); // 2026-06-08
    late.recalc(&RecalcContext::new(1_780_878_600_000 + 86_400_000, "Etc/GMT", 0).unwrap()); // +1 day
    assert_ne!(
        early.get("Sheet1", a1("A1")).unwrap().value(),
        late.get("Sheet1", a1("A1")).unwrap().value()
    );
}

#[test]
fn unknown_timezone_is_rejected() {
    assert!(RecalcContext::new(0, "Not/AZone", 0).is_none());
}

#[test]
fn rng_key_is_deterministic_and_position_sensitive() {
    let ctx = RecalcContext::new(0, "Etc/GMT", 42).unwrap();
    // Order-independent, identity-keyed: same coordinates ⇒ same key.
    assert_eq!(ctx.rng_key(0, 1, 1, 0), ctx.rng_key(0, 1, 1, 0));
    // Distinct cells / draws ⇒ distinct keys (with overwhelming probability).
    assert_ne!(ctx.rng_key(0, 1, 1, 0), ctx.rng_key(0, 1, 2, 0));
    assert_ne!(ctx.rng_key(0, 1, 1, 0), ctx.rng_key(0, 1, 1, 1));
    assert_ne!(ctx.rng_key(0, 1, 1, 0), ctx.rng_key(1, 1, 1, 0));
    // A different seed ⇒ a different key.
    let other = RecalcContext::new(0, "Etc/GMT", 43).unwrap();
    assert_ne!(ctx.rng_key(0, 1, 1, 0), other.rng_key(0, 1, 1, 0));
}
