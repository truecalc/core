//! `SPARKLINE` — behaviour pinned to the Google Sheets conformance fixtures.
//!
//! Every expectation below is a row of `tests/fixtures/google_sheets/google.tsv`,
//! observed in live Google Sheets.  Nothing here is self-confirmed.  The
//! fixture rows whose recorded value is the empty string (a rendered chart has
//! no text projection) are skipped by the TSV runner, so they are asserted
//! here instead — as "this renders, it is not an error", plus the parsed spec
//! the engine built.

mod helpers;
use helpers::{eval, eval_with};

use truecalc_core::types::{SparklineChartType, SparklineValue};
use truecalc_core::{CellAddr, Engine, ErrorKind, Ref, Resolver, Value};

/// The parsed spec of a formula that must evaluate to a sparkline.
fn spec(formula: &str) -> (SparklineChartType, Vec<SparklineValue>, Vec<(String, SparklineValue)>) {
    match eval(formula) {
        Value::Sparkline(s) => (s.chart_type, s.data.clone(), s.options.clone()),
        other => panic!("{formula} should render a sparkline, got {other:?}"),
    }
}

fn num(n: f64) -> SparklineValue {
    SparklineValue::Number(n)
}

// ── The result is a value kind of its own ───────────────────────────────────

#[test]
fn type_of_a_sparkline_is_128() {
    // google.tsv: =TYPE(SPARKLINE({1,2,3})) → 128, outside TYPE's documented
    // set (1 number / 2 text / 4 boolean / 16 error / 64 array).
    assert_eq!(eval("=TYPE(SPARKLINE({1,2,3}))"), Value::Number(128.0));
}

#[test]
fn a_sparkline_is_not_an_error() {
    // google.tsv: =ISERROR(SPARKLINE({1,2,3})) → FALSE, =ISNA(...) → FALSE.
    assert_eq!(eval("=ISERROR(SPARKLINE({1,2,3}))"), Value::Bool(false));
    assert_eq!(eval("=ISNA(SPARKLINE({1,2,3}))"), Value::Bool(false));
}

#[test]
fn every_sparkline_is_equal_to_every_other_sparkline() {
    // google.tsv — the row that reads like spec identity, plus the controls
    // that disprove it: `=` reports TRUE for two sparklines whatever they plot,
    // whatever their charttype, and whatever options they carry.
    for formula in [
        "=SPARKLINE({1,2,3})=SPARKLINE({1,2,3})",
        "=SPARKLINE({1,2,3})=SPARKLINE({9,9,9})",
        "=SPARKLINE({1,2,3},{\"charttype\",\"column\"})=SPARKLINE({1,2,3},{\"charttype\",\"line\"})",
        "=SPARKLINE({1,2,3},{\"charttype\",\"line\"})=SPARKLINE({1,2,3})",
        "=SPARKLINE({1,2,3},{\"bogus\",\"x\"})=SPARKLINE({1,2,3})",
        "=SPARKLINE({1,2,3},{\"bogus\",\"x\"})=SPARKLINE({1,2,3},{\"bogus\",\"y\"})",
    ] {
        assert_eq!(eval(formula), Value::Bool(true), "{formula}");
    }
    // …and `<>` is the negation of that, not of a spec comparison.
    assert_eq!(
        eval("=SPARKLINE({1,2,3})<>SPARKLINE({1,2,3})"),
        Value::Bool(false)
    );
}

#[test]
fn a_sparkline_is_not_equal_to_empty_text() {
    // google.tsv: `=SPARKLINE({1,2,3})=""` is FALSE — even though every text
    // projection of a sparkline is the empty string, and `EXACT(…,"")` is TRUE.
    assert_eq!(eval("=SPARKLINE({1,2,3})=\"\""), Value::Bool(false));
}

#[test]
fn a_sparkline_outranks_every_scalar_in_ordering() {
    // google.tsv: `>1`, `>"zzzz"` and `>TRUE` are all TRUE.
    for formula in [
        "=SPARKLINE({1,2,3})>1",
        "=SPARKLINE({1,2,3})>\"zzzz\"",
        "=SPARKLINE({1,2,3})>TRUE",
    ] {
        assert_eq!(eval(formula), Value::Bool(true), "{formula}");
    }
}

/// `EQ`/`NE`/`GT`/`GTE`/`LT`/`LTE` are Google Sheets' own function names for
/// `=`/`<>`/`>`/`>=`/`<`/`<=`, and the engine backs them with a *separate*
/// comparison path.  google.tsv records both paths agreeing (EQ TRUE, NE FALSE,
/// `<>` FALSE, GTE TRUE), so any divergence is an implementation defect.
#[test]
fn comparison_alias_functions_agree_with_their_operators() {
    let a = "SPARKLINE({1,2,3})";
    let b = "SPARKLINE({9,9,9})";
    let cases = [
        ("EQ", "="),
        ("NE", "<>"),
        ("GT", ">"),
        ("GTE", ">="),
        ("LT", "<"),
        ("LTE", "<="),
    ];
    for (func, op) in cases {
        for (left, right) in [(a, a), (a, b), (b, a)] {
            let via_function = eval(&format!("={func}({left},{right})"));
            let via_operator = eval(&format!("={left}{op}{right}"));
            assert_eq!(
                via_function, via_operator,
                "{func}({left},{right}) must match {left}{op}{right}"
            );
        }
    }
}

#[test]
fn eq_and_ne_agree_with_the_equality_operator() {
    // google.tsv records EQ TRUE and NE FALSE directly, reached through the
    // function alias rather than the operator.
    assert_eq!(
        eval("=EQ(SPARKLINE({1,2,3}),SPARKLINE({1,2,3}))"),
        Value::Bool(true)
    );
    assert_eq!(
        eval("=NE(SPARKLINE({1,2,3}),SPARKLINE({1,2,3}))"),
        Value::Bool(false)
    );
    // …and the same answers for two *different* sparklines, because `=` does
    // not look at what they plot.
    assert_eq!(
        eval("=EQ(SPARKLINE({1,2,3}),SPARKLINE({9,9,9}))"),
        Value::Bool(true)
    );
    assert_eq!(
        eval("=NE(SPARKLINE({1,2,3}),SPARKLINE({9,9,9}))"),
        Value::Bool(false)
    );
}

#[test]
fn two_sparklines_are_mutually_equal_in_ordering() {
    // google.tsv: `>` and `<` between two sparklines are both FALSE, while `>=`
    // (and its GTE alias, on two *different* sparklines) is TRUE — the ordering
    // reading of "all sparklines are equal".
    for formula in [
        "=SPARKLINE({1,2,3})>SPARKLINE({9,9,9})",
        "=SPARKLINE({1,2,3})<SPARKLINE({9,9,9})",
        "=GT(SPARKLINE({1,2,3}),SPARKLINE({9,9,9}))",
        "=LT(SPARKLINE({1,2,3}),SPARKLINE({9,9,9}))",
    ] {
        assert_eq!(eval(formula), Value::Bool(false), "{formula}");
    }
    for formula in [
        "=SPARKLINE({1,2,3})>=SPARKLINE({1,2,3})",
        "=SPARKLINE({1,2,3})>=SPARKLINE({9,9,9})",
        "=SPARKLINE({1,2,3})<=SPARKLINE({9,9,9})",
        "=GTE(SPARKLINE({1,2,3}),SPARKLINE({9,9,9}))",
    ] {
        assert_eq!(eval(formula), Value::Bool(true), "{formula}");
    }
}

// ── Coercion ────────────────────────────────────────────────────────────────
//
// Text and boolean contexts are permissive — a sparkline reads as empty text
// and as falsy — and aggregates skip it.  Exactly two contexts reject it: the
// arithmetic operators and the `&` concatenation *operator*.  Note that `&`
// errors while `CONCATENATE` of the same value succeeds; that asymmetry is
// recorded, not a modelling choice.

#[test]
fn coercion_arithmetic_on_a_sparkline_is_value_error() {
    // google.tsv: =SPARKLINE({1,2,3})+1 → #VALUE!
    assert_eq!(
        eval("=SPARKLINE({1,2,3})+1"),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn coercion_concatenating_a_sparkline_is_value_error() {
    // google.tsv: ="x"&SPARKLINE({1,2,3}) → #VALUE!  (empty text would have
    // concatenated instead.)
    assert_eq!(
        eval("=\"x\"&SPARKLINE({1,2,3})"),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn coercion_len_of_a_sparkline_is_zero() {
    // google.tsv: =LEN(SPARKLINE({1,2,3})) → 0
    assert_eq!(eval("=LEN(SPARKLINE({1,2,3}))"), Value::Number(0.0));
}

#[test]
fn coercion_n_of_a_sparkline_is_zero() {
    // google.tsv: =N(SPARKLINE({1,2,3})) → 0
    assert_eq!(eval("=N(SPARKLINE({1,2,3}))"), Value::Number(0.0));
}

#[test]
fn coercion_to_text_of_a_sparkline_is_the_empty_string() {
    // google.tsv: =TO_TEXT(SPARKLINE({1,2,3})) → "" (an empty recorded value,
    // so the TSV runner skips the row; asserted here instead).
    assert_eq!(
        eval("=TO_TEXT(SPARKLINE({1,2,3}))"),
        Value::Text(String::new())
    );
}

#[test]
fn coercion_text_functions_read_a_sparkline_as_empty_text() {
    // google.tsv: LEFT → "", TEXT → "", TEXTJOIN → "", CONCATENATE(…,"x") → "x",
    // EXACT(…,"") → TRUE.  The `&` operator above is the carve-out, not these.
    assert_eq!(
        eval("=LEFT(SPARKLINE({1,2,3}),1)"),
        Value::Text(String::new())
    );
    assert_eq!(
        eval("=TEXT(SPARKLINE({1,2,3}),\"0\")"),
        Value::Text(String::new())
    );
    assert_eq!(
        eval("=TEXTJOIN(\",\",TRUE,SPARKLINE({1,2,3}))"),
        Value::Text(String::new())
    );
    assert_eq!(
        eval("=CONCATENATE(SPARKLINE({1,2,3}),\"x\")"),
        Value::Text("x".to_owned())
    );
    assert_eq!(
        eval("=EXACT(SPARKLINE({1,2,3}),\"\")"),
        Value::Bool(true)
    );
}

#[test]
fn coercion_currency_and_conversion_functions_split_two_ways() {
    // google.tsv, and the reason the coercion map lists exceptions by name:
    // DOLLAR and FIXED reject a sparkline, while TEXT and the whole TO_* family
    // answer "".  DOLLAR vs TO_DOLLARS is the sharpest pair — near-identical
    // names, opposite answers.  (VALUE is not in that argument: it reads
    // through the permissive text seam, so its 0 comes free from LEN's route,
    // not from a number-reading carve-out.)
    assert_eq!(
        eval("=TO_PERCENT(SPARKLINE({1,2,3}))"),
        Value::Text(String::new())
    );
    assert_eq!(eval("=VALUE(SPARKLINE({1,2,3}))"), Value::Number(0.0));
    assert_eq!(
        eval("=DOLLAR(SPARKLINE({1,2,3}))"),
        Value::Error(ErrorKind::Value)
    );
    assert_eq!(
        eval("=FIXED(SPARKLINE({1,2,3}))"),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn coercion_the_whole_to_family_reads_a_sparkline_as_empty_text() {
    // google.tsv: TO_TEXT, TO_PERCENT, TO_DOLLARS, TO_PURE_NUMBER and TO_DATE
    // all answer "" — the family is uniform, which is exactly why TO_DOLLARS
    // parting company with DOLLAR is not derivable from either name.
    for formula in [
        "=TO_TEXT(SPARKLINE({1,2,3}))",
        "=TO_PERCENT(SPARKLINE({1,2,3}))",
        "=TO_DOLLARS(SPARKLINE({1,2,3}))",
        "=TO_PURE_NUMBER(SPARKLINE({1,2,3}))",
        "=TO_DATE(SPARKLINE({1,2,3}))",
    ] {
        assert_eq!(eval(formula), Value::Text(String::new()), "{formula}");
    }
}

#[test]
fn coercion_the_number_seam_rejects_a_sparkline_wholesale() {
    // `to_number` has one blanket arm, so this is not a list of four functions:
    // every caller of that seam rejects.  google.tsv pins the operators; these
    // are the probed function-level confirmations.
    for formula in [
        "=SPARKLINE({1,2,3})+1",
        "=SPARKLINE({1,2,3})-1",
        "=SPARKLINE({1,2,3})*2",
        "=SPARKLINE({1,2,3})/2",
        "=-SPARKLINE({1,2,3})",
        "=SPARKLINE({1,2,3})%",
        "=ROUND(SPARKLINE({1,2,3}))",
        "=ABS(SPARKLINE({1,2,3}))",
        "=INT(SPARKLINE({1,2,3}))",
    ] {
        assert_eq!(eval(formula), Value::Error(ErrorKind::Value), "{formula}");
    }
}

#[test]
fn coercion_more_text_functions_read_a_sparkline_as_empty_text() {
    // google.tsv: TRIM → "", UPPER → "".
    assert_eq!(
        eval("=TRIM(SPARKLINE({1,2,3}))"),
        Value::Text(String::new())
    );
    assert_eq!(
        eval("=UPPER(SPARKLINE({1,2,3}))"),
        Value::Text(String::new())
    );
}

#[test]
fn coercion_a_sparkline_is_falsy() {
    // google.tsv: =IF(SPARKLINE({1,2,3}),1,2) → 2.
    assert_eq!(eval("=IF(SPARKLINE({1,2,3}),1,2)"), Value::Number(2.0));
}

#[test]
fn coercion_aggregates_skip_a_sparkline_rather_than_erroring() {
    // google.tsv: SUM → 1, MAX → 1, PRODUCT(…,3) → 3, COUNT(…,1) → 1.  A
    // sparkline is skipped wherever it appears, direct argument or array
    // element — PRODUCT is the one that had a direct-argument path of its own.
    assert_eq!(eval("=SUM(SPARKLINE({1,2,3}),1)"), Value::Number(1.0));
    assert_eq!(eval("=MAX(SPARKLINE({1,2,3}),1)"), Value::Number(1.0));
    assert_eq!(eval("=PRODUCT(SPARKLINE({1,2,3}),3)"), Value::Number(3.0));
    assert_eq!(eval("=PRODUCT({SPARKLINE({1,2,3}),3})"), Value::Number(3.0));
    assert_eq!(eval("=COUNT(SPARKLINE({1,2,3}),1)"), Value::Number(1.0));
}

#[test]
fn the_statistical_family_skips_a_sparkline_too() {
    // google.tsv, recorded rather than assumed — MINA had no arm at all until a
    // row asked for one, and matching MAXA was a coin-flip until then.
    assert_eq!(eval("=MINA(SPARKLINE({1,2,3}))"), Value::Number(0.0));
    assert_eq!(eval("=MAXA(SPARKLINE({1,2,3}))"), Value::Number(0.0));
    assert_eq!(
        eval("=AVERAGEA(SPARKLINE({1,2,3}))"),
        Value::Error(ErrorKind::DivByZero)
    );
    assert_eq!(eval("=MEDIAN(SPARKLINE({1,2,3}),1)"), Value::Number(1.0));
    assert_eq!(
        eval("=STDEV(SPARKLINE({1,2,3}),1,2)"),
        Value::Number(0.7071067811865476)
    );
    assert_eq!(eval("=SUMSQ(SPARKLINE({1,2,3}))"), Value::Number(0.0));
    assert_eq!(eval("=SUMPRODUCT(SPARKLINE({1,2,3}))"), Value::Number(0.0));
    assert_eq!(
        eval("=CELL(\"type\",SPARKLINE({1,2,3}))"),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn a_lone_sparkline_leaves_an_aggregate_with_no_arguments_at_all() {
    // google.tsv: SUM, PRODUCT, MAX, MIN, MAXA, MINA and COUNT of nothing but a
    // sparkline are all 0 — PRODUCT included, so a skipped argument is *absent*,
    // not a factor of 1 (which two-argument calls cannot distinguish), and MIN
    // and the A-variants each have their own recorded direct-form row rather
    // than being inferred from the range form.  AVERAGE is #DIV/0!, the row that
    // proves the list is empty rather than holding a zero.
    for formula in [
        "=SUM(SPARKLINE({1,2,3}))",
        "=PRODUCT(SPARKLINE({1,2,3}))",
        "=MAX(SPARKLINE({1,2,3}))",
        "=MIN(SPARKLINE({1,2,3}))",
        "=MAXA(SPARKLINE({1,2,3}))",
        "=MINA(SPARKLINE({1,2,3}))",
        "=COUNT(SPARKLINE({1,2,3}))",
    ] {
        assert_eq!(eval(formula), Value::Number(0.0), "{formula}");
    }
    assert_eq!(
        eval("=AVERAGE(SPARKLINE({1,2,3}))"),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn a_sparkline_counts_as_a_present_non_blank_value() {
    // google.tsv: =COUNTA(SPARKLINE({1,2,3})) → 1, =ISBLANK(…) → FALSE.
    assert_eq!(eval("=COUNTA(SPARKLINE({1,2,3}))"), Value::Number(1.0));
    assert_eq!(eval("=ISBLANK(SPARKLINE({1,2,3}))"), Value::Bool(false));
}

#[test]
fn countunique_distinguishes_sparklines_that_the_equality_operator_does_not() {
    // google.tsv: 2 for two different sparklines, 1 for two identical ones —
    // even though `=` reports both pairs equal.  This is the row that requires
    // the parsed spec to be retained and serialized in full.
    assert_eq!(
        eval("=COUNTUNIQUE(SPARKLINE({1,2,3}),SPARKLINE({9,9,9}))"),
        Value::Number(2.0)
    );
    assert_eq!(
        eval("=COUNTUNIQUE(SPARKLINE({1,2,3}),SPARKLINE({1,2,3}))"),
        Value::Number(1.0)
    );
}

// ── Error class 1: arity / shape of `data` → #N/A ───────────────────────────

#[test]
fn error_na_when_called_with_no_arguments() {
    // google.tsv: =SPARKLINE() → #N/A
    assert_eq!(eval("=SPARKLINE()"), Value::Error(ErrorKind::NA));
}

#[test]
fn error_na_when_data_is_a_scalar_instead_of_a_range() {
    // google.tsv: =SPARKLINE(5) → #N/A
    assert_eq!(eval("=SPARKLINE(5)"), Value::Error(ErrorKind::NA));
}

#[test]
fn error_na_when_data_holds_a_single_value() {
    // google.tsv: =SPARKLINE({5}) → #N/A
    assert_eq!(eval("=SPARKLINE({5})"), Value::Error(ErrorKind::NA));
}

// ── Error class 2: structural malformation → #REF! ──────────────────────────

#[test]
fn error_ref_when_data_is_an_empty_array() {
    // google.tsv: =SPARKLINE({}) → #REF!  (Note this is *not* #N/A: an empty
    // array is malformed, a one-point array is merely too short.)
    assert_eq!(eval("=SPARKLINE({})"), Value::Error(ErrorKind::Ref));
}

#[test]
fn error_ref_when_options_are_not_key_value_pairs() {
    // google.tsv: =SPARKLINE({1,2,3},{"charttype"}) → #REF!
    assert_eq!(
        eval("=SPARKLINE({1,2,3},{\"charttype\"})"),
        Value::Error(ErrorKind::Ref)
    );
}

// ── Error class 3: a bad option *value* → #VALUE! ───────────────────────────

#[test]
fn error_value_when_the_charttype_value_is_unknown() {
    // google.tsv: =SPARKLINE({1,2,3},{"charttype","bogus"}) → #VALUE!
    assert_eq!(
        eval("=SPARKLINE({1,2,3},{\"charttype\",\"bogus\"})"),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn an_unknown_option_key_is_kept_not_rejected() {
    // google.tsv: =SPARKLINE({1,2,3},{"bogus","x"}) renders — an unrecognised
    // *key* is not an error, unlike an unrecognised charttype *value*.  Sheets
    // "ignores" it; the engine keeps it in the parsed spec, which `=` cannot
    // observe (all sparklines are equal) but COUNTUNIQUE's deeper key can — so
    // it is kept, treated exactly like a recognised option.
    let (chart_type, data, options) = spec("=SPARKLINE({1,2,3},{\"bogus\",\"x\"})");
    assert_eq!(chart_type, SparklineChartType::Line);
    assert_eq!(data, vec![num(1.0), num(2.0), num(3.0)]);
    assert_eq!(
        options,
        vec![("bogus".to_owned(), SparklineValue::Text("x".to_owned()))]
    );
}

// ── Cases that render ───────────────────────────────────────────────────────

#[test]
fn charttype_defaults_to_line_when_omitted() {
    // google.tsv: =SPARKLINE({1,2,3}) renders.
    let (chart_type, data, options) = spec("=SPARKLINE({1,2,3})");
    assert_eq!(chart_type, SparklineChartType::Line);
    assert_eq!(data, vec![num(1.0), num(2.0), num(3.0)]);
    assert!(options.is_empty());
}

#[test]
fn every_charttype_renders() {
    // google.tsv: column, bar (two values), winloss and line all render.
    let cases = [
        (
            "=SPARKLINE({1,2,3},{\"charttype\",\"column\"})",
            SparklineChartType::Column,
        ),
        (
            "=SPARKLINE({1,2},{\"charttype\",\"bar\"})",
            SparklineChartType::Bar,
        ),
        (
            "=SPARKLINE({1,-1,1},{\"charttype\",\"winloss\"})",
            SparklineChartType::Winloss,
        ),
        (
            "=SPARKLINE({1,2,3},{\"charttype\",\"line\";\"color\",\"red\"})",
            SparklineChartType::Line,
        ),
    ];
    for (formula, expected) in cases {
        let (chart_type, _, _) = spec(formula);
        assert_eq!(chart_type, expected, "{formula}");
    }
}

#[test]
fn bar_charttype_with_three_values_renders() {
    // google.tsv: =SPARKLINE({1,2,3},{"charttype","bar"}) renders, even though
    // `bar` is documented as two-valued.  A wrong-arity check here would be a
    // divergence, not a validation.
    let (chart_type, data, _) = spec("=SPARKLINE({1,2,3},{\"charttype\",\"bar\"})");
    assert_eq!(chart_type, SparklineChartType::Bar);
    assert_eq!(data.len(), 3);
}

#[test]
fn color_option_is_kept_and_charttype_is_lifted_out() {
    // google.tsv: =SPARKLINE({1,2,3},{"charttype","line";"color","red"}) renders.
    let (chart_type, _, options) =
        spec("=SPARKLINE({1,2,3},{\"charttype\",\"line\";\"color\",\"red\"})");
    assert_eq!(chart_type, SparklineChartType::Line);
    assert_eq!(
        options,
        vec![("color".to_owned(), SparklineValue::Text("red".to_owned()))]
    );
}

#[test]
fn ymin_and_ymax_options_render() {
    // google.tsv: =SPARKLINE({1,2,3},{"ymin",0;"ymax",10}) renders.
    let (_, _, options) = spec("=SPARKLINE({1,2,3},{\"ymin\",0;\"ymax\",10})");
    assert_eq!(
        options,
        vec![
            ("ymin".to_owned(), num(0.0)),
            ("ymax".to_owned(), num(10.0)),
        ]
    );
}

#[test]
fn text_inside_the_data_renders() {
    // google.tsv: =SPARKLINE({1,"a",3}) renders — text is a data point, not an
    // error.
    let (_, data, _) = spec("=SPARKLINE({1,\"a\",3})");
    assert_eq!(
        data,
        vec![num(1.0), SparklineValue::Text("a".to_owned()), num(3.0)]
    );
}

#[test]
fn all_negative_data_renders() {
    // google.tsv: =SPARKLINE({-1,-2,-3}) renders.
    let (_, data, _) = spec("=SPARKLINE({-1,-2,-3})");
    assert_eq!(data, vec![num(-1.0), num(-2.0), num(-3.0)]);
}

#[test]
fn a_genuine_blank_inside_the_range_renders() {
    // google.tsv: =SPARKLINE(Data!H1:H3) with H2 deliberately empty renders,
    // exactly like the all-present control =SPARKLINE(Data!I1:I3).  (The
    // neighbouring `{1,,3}` → #ERROR! row is Sheets rejecting the array-literal
    // syntax before SPARKLINE runs, not a blank-cell rule.)
    let blank = Value::Array(vec![Value::Number(1.0), Value::Empty, Value::Number(3.0)]);
    match eval_with("=SPARKLINE(RANGE)", [("RANGE", blank)]) {
        Value::Sparkline(s) => assert_eq!(
            s.data,
            vec![num(1.0), SparklineValue::Blank, num(3.0)],
            "a blank cell is a data point"
        ),
        other => panic!("a blank inside the range must still render, got {other:?}"),
    }
}

#[test]
fn an_invalid_array_literal_fails_before_sparkline_runs() {
    // google.tsv: =SPARKLINE({1,,3}) → #ERROR! in Sheets — an array-literal
    // parse failure, which this engine reports as #VALUE! (the code the
    // fixtures map #ERROR! onto).  Recorded here so the row is not mistaken
    // for a blank-cell rule; the row above is the trustworthy blank probe.
    assert_eq!(eval("=SPARKLINE({1,,3})"), Value::Error(ErrorKind::Value));
}

#[test]
fn two_dimensional_and_single_row_data_render() {
    // google.tsv: =SPARKLINE({1,2;3,4}) and =SPARKLINE({1,2}) both render.
    let (_, data, _) = spec("=SPARKLINE({1,2;3,4})");
    assert_eq!(data, vec![num(1.0), num(2.0), num(3.0), num(4.0)]);
    let (_, data, _) = spec("=SPARKLINE({1,2})");
    assert_eq!(data, vec![num(1.0), num(2.0)]);
}

#[test]
fn option_keys_and_charttype_values_are_case_insensitive() {
    // google.tsv: {"CHARTTYPE","column"} and {"charttype","COLUMN"} both render.
    let (chart_type, _, options) = spec("=SPARKLINE({1,2,3},{\"CHARTTYPE\",\"column\"})");
    assert_eq!(chart_type, SparklineChartType::Column);
    assert!(options.is_empty(), "an upper-case charttype key is still charttype");
    let (chart_type, _, _) = spec("=SPARKLINE({1,2,3},{\"charttype\",\"COLUMN\"})");
    assert_eq!(chart_type, SparklineChartType::Column);
}

#[test]
fn a_non_text_option_key_is_accepted() {
    // google.tsv: {TRUE,"x"} and {0,"x"} both render — a non-text key is not an
    // error.  (What the coerced key *string* is has not been probed; nothing
    // observable depends on it, since neither key is recognised.)
    for formula in [
        "=SPARKLINE({1,2,3},{TRUE,\"x\"})",
        "=SPARKLINE({1,2,3},{0,\"x\"})",
    ] {
        let (chart_type, _, options) = spec(formula);
        assert_eq!(chart_type, SparklineChartType::Line, "{formula}");
        assert_eq!(options.len(), 1, "{formula}");
    }
}

// ── Delivery through a range ────────────────────────────────────────────────
//
// A real workbook delivers a sparkline from a *cell* holding `=SPARKLINE(...)`,
// not as a literal argument.  google.tsv probes that with `Data!K1` (a
// sparkline cell) and `Data!K2` (the number 5).  The shared TSV runner
// evaluates rows standalone with no workbook behind it, so those rows are
// skipped there — and would silently pass for the wrong reason where an empty
// read happens to match — which is why they are enforced here against a
// resolver seeded with exactly those two cells.

/// The `Data` sheet google.tsv's `Data!K…` rows read: K1 holds a sparkline,
/// K2 holds 5.
struct SparklineCellResolver;

impl SparklineCellResolver {
    fn cell(addr: &CellAddr) -> Value {
        match (addr.col, addr.row) {
            // K1 = `=SPARKLINE({1,2,3})`, pre-resolved as a workbook would.
            (11, 1) => eval("=SPARKLINE({1,2,3})"),
            (11, 2) => Value::Number(5.0),
            _ => Value::Empty,
        }
    }
}

impl Resolver for SparklineCellResolver {
    fn resolve(&mut self, r: &Ref) -> Value {
        match r {
            Ref::Cell { sheet: Some(s), addr } if s.eq_ignore_ascii_case("data") => {
                Self::cell(addr)
            }
            Ref::Range { sheet: Some(s), start, end } if s.eq_ignore_ascii_case("data") => {
                let mut cells = Vec::new();
                for row in start.row..=end.row {
                    for col in start.col..=end.col {
                        cells.push(Self::cell(&CellAddr { col, row, ..*start }));
                    }
                }
                Value::Array(cells)
            }
            _ => Value::Error(ErrorKind::Ref),
        }
    }
}

fn eval_over_cells(formula: &str) -> Value {
    Engine::sheets().evaluate_with_resolver(formula, &mut SparklineCellResolver)
}

#[test]
fn a_referenced_sparkline_cell_is_still_a_sparkline() {
    // google.tsv: =TYPE(Data!K1) → 128.  Reading a cell is not a coercion
    // point, even though the cell displays as empty.
    assert_eq!(eval_over_cells("=TYPE(Data!K1)"), Value::Number(128.0));
    assert!(matches!(
        eval_over_cells("=Data!K1"),
        Value::Sparkline(_)
    ));
}

#[test]
fn aggregates_answer_the_same_whether_a_sparkline_arrives_directly_or_by_range() {
    // google.tsv, K1:K1 (a lone sparkline cell) and K1:K2 (sparkline + 5).
    // The lone-cell rows are the evidence: with a second value present,
    // "skipped" and "contributes the identity" are indistinguishable for
    // PRODUCT (1 × 5 = 5 either way).
    for (formula, expected) in [
        // A lone sparkline cell: every aggregate answers 0, whichever way it
        // arrived. MAX and MAXA/MINA are the ones that had to change — MAX fell
        // into its numberless-array `#REF!` rule, and the A-variants into their
        // own `#N/A`.
        ("=SUM(Data!K1:K1)", 0.0),
        ("=PRODUCT(Data!K1:K1)", 0.0),
        ("=MAX(Data!K1:K1)", 0.0),
        ("=MIN(Data!K1:K1)", 0.0),
        ("=MAXA(Data!K1:K1)", 0.0),
        ("=MINA(Data!K1:K1)", 0.0),
        // …and with a real value alongside it, the sparkline is simply absent.
        ("=SUM(Data!K1:K2)", 5.0),
        ("=PRODUCT(Data!K1:K2)", 5.0),
        ("=MAX(Data!K1:K2)", 5.0),
        ("=MIN(Data!K1:K2)", 5.0),
        ("=MAXA(Data!K1:K2)", 5.0),
        ("=COUNT(Data!K1:K2)", 1.0),
        ("=COUNTA(Data!K1:K2)", 2.0),
        ("=COUNTUNIQUE(Data!K1:K2)", 2.0),
    ] {
        assert_eq!(
            eval_over_cells(formula),
            Value::Number(expected),
            "{formula}"
        );
    }
    // AVERAGE is the one exception, and a confirming one: an argument list that
    // empties rather than gaining a zero.
    assert_eq!(
        eval_over_cells("=AVERAGE(Data!K1:K1)"),
        Value::Error(ErrorKind::DivByZero)
    );
    // …and the direct forms agree with the range forms.
    assert_eq!(eval("=SUM(SPARKLINE({1,2,3}))"), eval_over_cells("=SUM(Data!K1:K1)"));
    assert_eq!(
        eval("=PRODUCT(SPARKLINE({1,2,3}))"),
        eval_over_cells("=PRODUCT(Data!K1:K1)")
    );
    for (direct, ranged) in [
        ("=MAX(SPARKLINE({1,2,3}))", "=MAX(Data!K1:K1)"),
        ("=MIN(SPARKLINE({1,2,3}))", "=MIN(Data!K1:K1)"),
        ("=MAXA(SPARKLINE({1,2,3}))", "=MAXA(Data!K1:K1)"),
        ("=MINA(SPARKLINE({1,2,3}))", "=MINA(Data!K1:K1)"),
        ("=AVERAGE(SPARKLINE({1,2,3}))", "=AVERAGE(Data!K1:K1)"),
    ] {
        assert_eq!(eval(direct), eval_over_cells(ranged), "{direct} vs {ranged}");
    }
}

#[test]
fn an_empty_array_argument_outranks_the_sparkline_skip() {
    // google.tsv: `=MAX(SPARKLINE({1,2,3}),{})` is #REF! while
    // `=MAX(SPARKLINE({1,2,3}),{"a"})` is 0.  So "a skipped sparkline leaves an
    // empty argument list ⇒ 0" holds against a text-only array but not against
    // an explicitly empty one, which raises MAX's own error first.
    assert_eq!(
        eval("=MAX(SPARKLINE({1,2,3}),{})"),
        Value::Error(ErrorKind::Ref)
    );
    assert_eq!(
        eval("=MAX(SPARKLINE({1,2,3}),{\"a\"})"),
        Value::Number(0.0)
    );
    // bugs.tsv: `=MIN(SPARKLINE({1,2,3}),{})` is #REF! — a live Google Sheets
    // value this engine used to miss. MIN now carries the same empty-array
    // rule, so its counterpart row agrees with MAX's.
    assert_eq!(
        eval("=MIN(SPARKLINE({1,2,3}),{})"),
        Value::Error(ErrorKind::Ref)
    );
}

// ── Registry surface ────────────────────────────────────────────────────────

#[test]
fn sparkline_is_listed_by_the_function_registry() {
    let registry = truecalc_core::Registry::new();
    let meta = registry
        .get_metadata()
        .into_iter()
        .find(|e| e.name == "SPARKLINE")
        .expect("SPARKLINE should be listed by the registry");
    assert_eq!(meta.meta.category, "google");
    assert_eq!(meta.meta.signature, "SPARKLINE(data, [options])");
}
