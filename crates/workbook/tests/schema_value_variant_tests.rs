//! Every `Value` variant's *serialized* form validates against the committed
//! `schema/workbook.v1.schema.json` (issue #768).
//!
//! The two branches that were missing — `zoned` and `sparkline` — are the
//! symptom; this file is the fix. A variant added to `Value` without a matching
//! schema branch fails here twice over:
//!
//! 1. [`variant_name`] matches exhaustively on `Value`, so a new variant stops
//!    this test crate compiling until it is named.
//! 2. [`declared_variants`] reads the variant list out of `src/value.rs`
//!    itself, so [`the_sample_set_covers_every_declared_variant`] fails until a
//!    sample is constructed — and the sample is then serialized and validated
//!    by [`every_variant_validates_against_the_schema`], which fails until the
//!    schema describes it.
//!
//! The two gates are not fully redundant: gate 2 recognises only the tuple and
//! unit forms (`Name(..)` / `Name,`), so a struct-form variant would slip past
//! it and be caught by gate 1 alone. Gate 1 covers every shape, so nothing
//! escapes both — but gate 2 is the one that survives a careless `_ =>` arm,
//! and it does not see a struct variant.
//!
//! Samples are checked as *serializer output*, never as hand-written JSON: the
//! schema has to describe what the serializer emits, and the two drift silently
//! otherwise.

use std::collections::BTreeSet;

use chrono_tz::TZ_VARIANTS;
use truecalc_core::types::sparkline::{SparklineChartType, SparklineSpec, SparklineValue};
use truecalc_core::types::zoned::{ZoneId, ZonedInstant};
use truecalc_workbook::{Cell, EngineFlavor, Value, Workbook, Worksheet};

fn validator() -> jsonschema::Validator {
    let text = std::fs::read_to_string("schema/workbook.v1.schema.json").unwrap();
    jsonschema::validator_for(&serde_json::from_str(&text).unwrap()).expect("schema compiles")
}

/// The variant this value belongs to.
///
/// Exhaustive on purpose: adding a variant to `Value` makes this `match`
/// non-exhaustive and the test crate stops compiling.
fn variant_name(value: &Value) -> &'static str {
    match value {
        Value::Number(_) => "Number",
        Value::Text(_) => "Text",
        Value::Boolean(_) => "Boolean",
        Value::Error(_) => "Error",
        Value::ErrorMsg(_, _) => "ErrorMsg",
        Value::Empty => "Empty",
        Value::Array(_) => "Array",
        Value::Date(_) => "Date",
        Value::Zoned(_) => "Zoned",
        Value::Sparkline(_) => "Sparkline",
    }
}

/// The variant names declared by `pub enum Value`, read out of the source so
/// the sample set below cannot silently fall behind the type.
fn declared_variants() -> BTreeSet<String> {
    const SRC: &str = include_str!("../src/value.rs");
    let body = SRC
        .split_once("pub enum Value {")
        .expect("src/value.rs declares `pub enum Value`")
        .1
        .split_once("\n}")
        .expect("the enum body is closed by a brace in column 0")
        .0;
    body.lines()
        .filter_map(|line| {
            // A variant is a single-indent line `Name,` or `Name(..)`; doc
            // comments and attributes fail the trailing-delimiter check.
            let line = line.strip_prefix("    ")?;
            let end = line.find(|c: char| !c.is_alphanumeric() && c != '_')?;
            let (name, rest) = line.split_at(end);
            (!name.is_empty() && rest.starts_with(['(', ','])).then(|| name.to_owned())
        })
        .collect()
}

fn zoned(utc_nanos: i64, zone: ZoneId) -> Value {
    Value::Zoned(Box::new(ZonedInstant::from_instant(utc_nanos, zone)))
}

/// One representative instance of every `Value` variant.
fn samples() -> Vec<Value> {
    vec![
        Value::Number(1.5),
        Value::Text("hi".to_owned()),
        Value::Boolean(true),
        Value::Error("#REF!".to_owned()),
        Value::ErrorMsg("#VALUE!".to_owned(), "a diagnostic".to_owned()),
        Value::Empty,
        Value::Array(vec![vec![Value::Number(1.0), Value::Text("a".to_owned())]]),
        Value::Date(46180.5),
        zoned(
            1_768_000_000_000_000_000,
            ZoneId::Iana("Europe/Berlin".parse().unwrap()),
        ),
        Value::Sparkline(Box::new(SparklineSpec {
            chart_type: SparklineChartType::Column,
            data: vec![
                SparklineValue::number(1.0),
                SparklineValue::Blank,
                SparklineValue::Text("a".to_owned()),
                SparklineValue::Bool(false),
            ],
            options: vec![
                ("color".to_owned(), SparklineValue::Text("red".to_owned())),
                ("ymin".to_owned(), SparklineValue::number(0.0)),
            ],
        })),
    ]
}

/// A one-cell workbook holding `value`. The cell carries a formula because
/// `Value::Empty` is legal only as an unevaluated formula cell's result.
fn workbook_holding(value: Value) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    let mut sheet = Worksheet::new("S");
    sheet
        .cells_mut()
        .insert("A1".to_owned(), Cell::with_formula("=1", value));
    wb.sheets_mut().push(sheet);
    wb
}

#[test]
fn the_sample_set_covers_every_declared_variant() {
    let covered: BTreeSet<String> = samples()
        .iter()
        .map(|v| variant_name(v).to_owned())
        .collect();
    assert_eq!(
        covered,
        declared_variants(),
        "every `Value` variant needs a sample here, so that its serialized form \
         is checked against the published schema"
    );
}

#[test]
fn every_variant_validates_against_the_schema() {
    let validator = validator();
    for value in samples() {
        let name = variant_name(&value).to_owned();
        let text = workbook_holding(value).to_json().unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(
            validator.is_valid(&json),
            "the serialized form of `Value::{name}` does not validate against \
             the published schema: {text}"
        );
    }
}

#[test]
fn every_variant_round_trips_and_still_validates() {
    let validator = validator();
    for value in samples() {
        let name = variant_name(&value).to_owned();
        let text = workbook_holding(value).to_json().unwrap();
        let reparsed = Workbook::from_json(text.as_bytes())
            .unwrap_or_else(|e| panic!("`Value::{name}` failed to reparse: {e:?} ({text})"));
        let again = reparsed.to_json().unwrap();
        assert_eq!(again, text, "`Value::{name}` did not round-trip");
        assert!(
            validator.is_valid(&serde_json::from_str(&again).unwrap()),
            "the re-serialized form of `Value::{name}` does not validate against \
             the published schema: {again}"
        );
    }
}

#[test]
fn every_scalar_variant_validates_inside_an_array() {
    // A spill anchor's array holds scalars only — a nested array is
    // unrepresentable — so the `Array` sample is the one exclusion.
    let validator = validator();
    for value in samples() {
        if matches!(value, Value::Array(_)) {
            continue;
        }
        let name = variant_name(&value).to_owned();
        let row = Value::Array(vec![vec![value, Value::Number(0.0)]]);
        let text = workbook_holding(row).to_json().unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(
            validator.is_valid(&json),
            "`Value::{name}` inside a spilled array does not validate against \
             the published schema: {text}"
        );
    }
}

#[test]
fn the_zoned_branch_accepts_every_zone_in_the_pinned_tzdb() {
    // The branch's string pattern is only as good as the zone names it has
    // seen; check it against every name the serializer can actually emit, at
    // two instants six months apart so both DST states are exercised.
    let validator = validator();
    for tz in TZ_VARIANTS {
        for utc_nanos in [1_767_225_600_000_000_000, 1_782_950_400_000_000_000] {
            let text = workbook_holding(zoned(utc_nanos, ZoneId::Iana(tz)))
                .to_json()
                .unwrap();
            assert!(
                validator.is_valid(&serde_json::from_str(&text).unwrap()),
                "a zoned value in {} does not validate: {text}",
                tz.name()
            );
        }
    }
    // Fixed-offset zones drop the bracketed name entirely.
    for minutes in [-720, -330, 0, 330, 840] {
        let text = workbook_holding(zoned(0, ZoneId::Fixed(minutes)))
            .to_json()
            .unwrap();
        assert!(
            validator.is_valid(&serde_json::from_str(&text).unwrap()),
            "a fixed-offset zoned value does not validate: {text}"
        );
    }
}

/// The zoned pattern must not be narrower than the reader either: the reader
/// takes any RFC-3339 spelling and normalizes it on the way in, so a document
/// spelling an instant that way is loadable and must not be called malformed.
#[test]
fn the_zoned_branch_accepts_the_spellings_the_reader_accepts() {
    let validator = validator();
    for value in [
        "2026-01-01T12:00:00Z",
        "2026-01-01T12:00:00Z[UTC]",
        "2026-01-01T12:00:00.5+02:00",
        "2026-01-01T12:00:00+02:00[+02:00]",
        // A lower-case separator, a space separator, and a leap second are all
        // RFC-3339 spellings the reader normalizes on the way in.
        "2026-01-01t12:00:00z",
        "2026-01-01 12:00:00Z",
        "2016-12-31T23:59:60Z",
        // The reader's offset bound is +/-23:59, not the +/-14:00 of a real zone.
        "2026-01-01T12:00:00+23:59",
        "2026-01-01T12:00:00-00:00",
        "2026-01-01T12:00:00+02:00[Etc/GMT+5]",
        "2026-01-01T12:00:00+02:00[America/Port-au-Prince]",
    ] {
        let text = document_with_value(&format!(r#"{{"type":"zoned","value":"{value}"}}"#));
        assert!(
            Workbook::from_json(text.as_bytes()).is_ok(),
            "the reader should have accepted {value}"
        );
        assert!(
            validator.is_valid(&serde_json::from_str(&text).unwrap()),
            "the schema should have accepted {value}"
        );
    }
}

/// A one-cell document whose only cell carries the given raw JSON value.
fn document_with_value(value: &str) -> String {
    format!(
        r#"{{"engine":"sheets","names":[],"sheets":[{{"cells":{{"A1":{{"formula":"=1","value":{value}}}}},"name":"S"}}],"version":"1"}}"#
    )
}

/// The new branches must reject what the deserializer rejects — a schema
/// looser than `Workbook::from_json` mis-tells a consumer that a document it
/// cannot load is fine, which is the same defect as #768 with the sign flipped.
#[test]
fn the_new_branches_reject_what_the_deserializer_rejects() {
    let bad = [
        r#"{"type":"zoned","value":"not a timestamp"}"#,
        // RFC-3339 requires an offset; the reader refuses a bare wall clock.
        r#"{"type":"zoned","value":"2026-01-01T12:00:00"}"#,
        r#"{"type":"zoned","value":"2026-01-01T12:00:00+02:00[Europe/Berlin"}"#,
        // Out-of-range fields. The pattern bounds every field it can; calendar
        // validity (`2026-04-31`) and the representable instant range
        // (`9999-01-01`) are the two it genuinely cannot, and the branch's
        // description says so.
        r#"{"type":"zoned","value":"2026-99-99T99:99:99+99:99"}"#,
        r#"{"type":"zoned","value":"2026-01-01T12:00:00+24:00"}"#,
        // Whitespace padding: the wire is canonical-only, so the reader refuses
        // to absorb it even though the formula-level parser trims.
        r#"{"type":"zoned","value":" 2026-01-01T12:00:00Z"}"#,
        r#"{"type":"zoned","value":"2026-01-01T12:00:00Z\n"}"#,
        r#"{"type":"zoned","value":"2026-01-01T12:00:00+02:00[ Europe/Berlin ]"}"#,
        // A charttype is canonical lower-case on the wire, even though
        // SPARKLINE's own argument matching is case-insensitive.
        r#"{"type":"sparkline","value":{"charttype":"Line","data":[{"type":"number","value":1},{"type":"number","value":2}],"options":[]}}"#,
        r#"{"type":"sparkline","value":{"charttype":"WINLOSS","data":[{"type":"number","value":1},{"type":"number","value":2}],"options":[]}}"#,
        // A single data point is unrepresentable (the evaluator answers #N/A).
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1}],"options":[]}}"#,
        r#"{"type":"sparkline","value":{"charttype":"bogus","data":[{"type":"number","value":1},{"type":"number","value":2}],"options":[]}}"#,
        // `options` is not optional on the wire, even when empty.
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1},{"type":"number","value":2}]}}"#,
        // Option keys are stored ASCII-lower-cased.
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1},{"type":"number","value":2}],"options":[["COLOR",{"type":"text","value":"red"}]]}}"#,
        // charttype has its own field and is never repeated in the options.
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1},{"type":"number","value":2}],"options":[["charttype",{"type":"text","value":"bar"}]]}}"#,
        // An option is a [key, value] pair, and a point is a scalar value.
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1},{"type":"number","value":2}],"options":[["color"]]}}"#,
        r#"{"type":"sparkline","value":{"charttype":"line","data":[{"type":"number","value":1},{"type":"date","value":2}],"options":[]}}"#,
    ];
    let validator = validator();
    for value in bad {
        let text = document_with_value(value);
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(
            !validator.is_valid(&json),
            "the schema should have rejected {value}"
        );
        assert!(
            Workbook::from_json(text.as_bytes()).is_err(),
            "the deserializer should have rejected {value}"
        );
    }
}
