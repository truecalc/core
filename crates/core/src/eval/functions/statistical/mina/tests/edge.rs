use super::super::mina_fn;
use crate::eval::functions::statistical::min::min_fn;
use crate::types::{ErrorKind, Value, ZoneId, ZonedInstant};

fn zoned(utc_nanos: i64) -> Value {
    Value::Zoned(Box::new(ZonedInstant::from_instant(
        utc_nanos,
        ZoneId::Iana(chrono_tz::Tz::UTC),
    )))
}

#[test]
fn empty_values_skipped() {
    // Empty is skipped; min of remaining values
    assert_eq!(
        mina_fn(&[Value::Empty, Value::Number(7.0), Value::Empty]),
        Value::Number(7.0)
    );
}

#[test]
fn single_true_returns_one() {
    // Only TRUE=1 → result is 1.0
    assert_eq!(mina_fn(&[Value::Bool(true)]), Value::Number(1.0));
}

#[test]
fn negative_numbers_min() {
    assert_eq!(
        mina_fn(&[Value::Number(-3.0), Value::Number(-1.0), Value::Number(-5.0)]),
        Value::Number(-5.0)
    );
}

#[test]
fn bool_and_number_mixed() {
    // TRUE=1 and FALSE=0 mixed with numbers: min(1, 0, 10, 0) = 0
    assert_eq!(
        mina_fn(&[
            Value::Bool(true),
            Value::Bool(false),
            Value::Number(10.0),
            Value::Number(0.0)
        ]),
        Value::Number(0.0)
    );
}

#[test]
fn mina_empty_array_is_ref_error() {
    // `=MINA({})` is #REF!, as it is for MIN and MAX. Reached by the
    // empty-argument check alone: text folds in as 0 here, so a populated
    // array is never numberless in the first place.
    assert_eq!(
        mina_fn(&[Value::Array(vec![])]),
        Value::Error(ErrorKind::Ref)
    );
    assert_eq!(
        mina_fn(&[Value::Number(1.0), Value::Array(vec![])]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn mina_array_of_only_blanks_is_zero() {
    // `=MINA(A1:A3)` over empty cells is 0 in Google Sheets — the same answer
    // MAX, MIN and MAXA give it. MINA used to answer #N/A here.
    //
    // Captured across seven range shapes, each with a populated control, but
    // **none of those rows are in this repo yet** — they land in a separate
    // fixtures-only PR (see `stat_helpers::is_blank_only_array` for the shapes
    // and the branch). Read from this repo alone, this test pins the
    // behaviour, not the Sheets answer.
    assert_eq!(
        mina_fn(&[Value::Array(vec![Value::Empty, Value::Empty, Value::Empty])]),
        Value::Number(0.0)
    );
    // Same through the nested-row shape a vertical range materializes as.
    assert_eq!(
        mina_fn(&[Value::Array(vec![
            Value::Array(vec![Value::Empty]),
            Value::Array(vec![Value::Empty]),
        ])]),
        Value::Number(0.0)
    );
}

#[test]
fn mina_blank_without_an_array_is_still_na() {
    // The rule is confined to the range form. A bare blank argument is
    // unprobed, so it keeps the #N/A MINA has always given it.
    assert_eq!(mina_fn(&[Value::Empty]), Value::Error(ErrorKind::NA));
    assert_eq!(mina_fn(&[Value::Empty, Value::Empty]), Value::Error(ErrorKind::NA));
}

#[test]
fn mina_text_only_array_is_zero() {
    // `=MINA({"a","b"})` is 0 — text counts as zero rather than being
    // skipped, so this needs no separate rule.
    assert_eq!(
        mina_fn(&[Value::Array(vec![
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
        ])]),
        Value::Number(0.0)
    );
}

#[test]
fn dates_participate_and_type_the_answer() {
    // MINA agrees with MIN on every date input: a date-only array answers the
    // smallest serial (it was #N/A before dates were captured), a small plain
    // number beats every date, and the answer is date-typed whenever a date
    // took part — including when the plain number won.
    assert_eq!(
        mina_fn(&[Value::Array(vec![
            Value::Date(43831.0),
            Value::Date(44197.0)
        ])]),
        Value::Date(43831.0)
    );
    assert_eq!(
        mina_fn(&[Value::Date(43831.0), Value::Date(44197.0)]),
        Value::Date(43831.0)
    );
    assert_eq!(
        mina_fn(&[Value::Array(vec![Value::Date(43831.0), Value::Number(5.0)])]),
        Value::Date(5.0)
    );
    // A nested-row range materialization takes the same path.
    assert_eq!(
        mina_fn(&[Value::Array(vec![Value::Array(vec![Value::Date(43831.0)])])]),
        Value::Date(43831.0)
    );
    // No date in scope: still a plain number.
    assert_eq!(
        mina_fn(&[Value::Number(5.0), Value::Bool(true)]),
        Value::Number(1.0)
    );
    // An all-blank array is a separate, captured case and answers 0, not the
    // date rule and not #N/A — see `mina_array_of_only_blanks_is_zero`. Kept
    // here as the contrast: a date in scope types the answer, an array with
    // nothing in it but blanks does not.
    assert_eq!(
        mina_fn(&[Value::Array(vec![Value::Empty, Value::Empty])]),
        Value::Number(0.0)
    );
}

/// Regression for #781: MINA answers a zone-aware value exactly as MIN does.
///
/// `Zoned` is a truecalc extension with no Google Sheets equivalent, so the
/// conformance oracle cannot settle this and no captured row exists. The rule
/// below is therefore a **deliberate truecalc-only decision**, recorded on
/// [`mina_fn`] with its reasoning — not an observed Sheets answer.
///
/// What it pins is agreement: whatever the pair answers, it answers the same
/// thing. MINA used to drop the zoned value silently and hand back a
/// confident number where MIN errored.
#[test]
fn zoned_values_take_the_same_route_as_min() {
    let earlier = zoned(0);
    let later = zoned(1_000_000_000);

    // A zoned instant beside a naive serial is #VALUE! — the answer MIN has
    // always given it, now MINA's too.
    let mixed = [earlier.clone(), Value::Date(43831.0)];
    assert_eq!(mina_fn(&mixed), Value::Error(ErrorKind::Value));
    assert_eq!(mina_fn(&mixed), min_fn(&mixed));

    let mixed_number = [earlier.clone(), Value::Number(5.0)];
    assert_eq!(mina_fn(&mixed_number), Value::Error(ErrorKind::Value));
    assert_eq!(mina_fn(&mixed_number), min_fn(&mixed_number));

    // MINA-only coercions do not create an exception: a boolean and text both
    // contribute a number here, so both still collide with the zoned value.
    let mixed_bool = [earlier.clone(), Value::Bool(true)];
    assert_eq!(mina_fn(&mixed_bool), Value::Error(ErrorKind::Value));
    assert_eq!(mina_fn(&mixed_bool), min_fn(&mixed_bool));
    let mixed_text = [
        Value::Array(vec![earlier.clone(), Value::Text("a".to_string())]),
    ];
    assert_eq!(mina_fn(&mixed_text), Value::Error(ErrorKind::Value));
    assert_eq!(mina_fn(&mixed_text), min_fn(&mixed_text));

    // Zoned values on their own compare as instants and answer the earliest,
    // keeping its own zone — again the same as MIN.
    let zoned_only = [earlier.clone(), later.clone()];
    assert_eq!(mina_fn(&zoned_only), earlier);
    assert_eq!(mina_fn(&zoned_only), min_fn(&zoned_only));

    // Through an array (the shape a range materializes as) too.
    let through_range = [Value::Array(vec![earlier.clone(), later])];
    assert_eq!(mina_fn(&through_range), earlier);
    assert_eq!(mina_fn(&through_range), min_fn(&through_range));

    // An error still wins over the zoned rule, as it does for MIN.
    let with_error = [earlier.clone(), Value::Error(ErrorKind::NA)];
    assert_eq!(mina_fn(&with_error), Value::Error(ErrorKind::NA));
    assert_eq!(mina_fn(&with_error), min_fn(&with_error));

    // The zoned check sits before the argument loop, so it precedes the
    // empty-array #REF!, the sparkline-only 0 and the blank-only 0. Those three
    // answers move when a zoned instant is in scope — deliberately, because it
    // is the ordering MIN has always had. Pinned so a later reordering is a
    // test failure, not a surprise.
    let with_empty_array = [earlier.clone(), Value::Array(vec![])];
    assert_eq!(mina_fn(&with_empty_array), earlier);
    assert_eq!(mina_fn(&with_empty_array), min_fn(&with_empty_array));

    let with_blank_array = [earlier.clone(), Value::Array(vec![Value::Empty])];
    assert_eq!(mina_fn(&with_blank_array), earlier);
    assert_eq!(mina_fn(&with_blank_array), min_fn(&with_blank_array));

    // And a zoned instant on its own is the zoned instant, where MINA used to
    // answer #N/A.
    assert_eq!(mina_fn(&[earlier.clone()]), earlier);
    assert_eq!(mina_fn(&[earlier.clone()]), min_fn(&[earlier]));
}
