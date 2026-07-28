//! The parsed, validated render spec produced by `SPARKLINE`.
//!
//! Google Sheets models a sparkline as a **distinct value kind**, not as a
//! specially-formatted string: `=TYPE(SPARKLINE({1,2,3}))` returns `128`,
//! which is outside `TYPE`'s documented set (1 number / 2 text / 4 boolean /
//! 16 error / 64 array), and `=ISERROR(SPARKLINE({1,2,3}))` is `FALSE`.  That
//! is why [`crate::types::Value`] gains its own variant rather than folding
//! the result into `Text` or an error — see the Google Sheets conformance
//! fixtures (`tests/fixtures/google_sheets/google.tsv`).
//!
//! The spec is **not** what the `=` operator compares: Sheets reports *any* two
//! sparklines equal, whatever they plot (`=SPARKLINE({1,2,3})=SPARKLINE({9,9,9})`
//! is `TRUE`, and so is the same row across differing charttypes and options).
//! In that respect it follows the `ErrorMsg` precedent, whose payload is also
//! excluded from equality.
//!
//! The spec is still carried in full by every surface, because `COUNTUNIQUE`
//! *does* distinguish two different sparklines (2 for different, 1 for
//! identical) — Sheets keys uniqueness off something deeper than `=` compares,
//! so a lossy serialization would break that instead.

/// The `charttype` of a sparkline.  `line` is the default when the option is
/// omitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SparklineChartType {
    Line,
    Bar,
    Column,
    Winloss,
}

impl SparklineChartType {
    /// The lower-case wire name, as written in the `charttype` option.
    pub fn as_str(self) -> &'static str {
        match self {
            SparklineChartType::Line => "line",
            SparklineChartType::Bar => "bar",
            SparklineChartType::Column => "column",
            SparklineChartType::Winloss => "winloss",
        }
    }

    /// Parse a `charttype` option value.  Matching is ASCII case-insensitive.
    /// An unrecognised value is an error in Sheets (`#VALUE!`), unlike an
    /// unrecognised option *key*, which is silently ignored.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "line" => Some(SparklineChartType::Line),
            "bar" => Some(SparklineChartType::Bar),
            "column" => Some(SparklineChartType::Column),
            "winloss" => Some(SparklineChartType::Winloss),
            _ => None,
        }
    }
}

/// One plotted data point, or one option value.
///
/// A blank cell inside the source range is a legitimate data point (it renders
/// normally — the fixtures probe a real range with a deliberately empty cell),
/// so `Blank` is a value here, not an error.  Text inside the data likewise
/// renders.
#[derive(Debug, Clone, PartialEq)]
pub enum SparklineValue {
    /// A finite number.  `-0.0` is normalized to `0.0` on construction so that
    /// structural equality and any hash of the spec agree.
    Number(f64),
    Text(String),
    Bool(bool),
    /// An empty cell.
    Blank,
}

impl SparklineValue {
    /// Build a numeric point, normalizing `-0.0` to `0.0`.
    pub fn number(n: f64) -> Self {
        SparklineValue::Number(if n == 0.0 { 0.0 } else { n })
    }
}

/// A parsed, validated sparkline render spec: what to plot and how.
///
/// Drawing is the consumer's job; the engine's job is to parse, validate and
/// carry this faithfully across every surface.
#[derive(Debug, Clone, PartialEq)]
pub struct SparklineSpec {
    /// The chart type (`line` when the option is omitted).
    pub chart_type: SparklineChartType,
    /// The points to plot, flattened row-major from the `data` argument.
    pub data: Vec<SparklineValue>,
    /// The remaining option key/value pairs in the order given, keys
    /// ASCII-lower-cased.  `charttype` is lifted into [`Self::chart_type`] and
    /// is not repeated here.  Keys the engine does not recognise are kept
    /// rather than rejected: Sheets ignores an unknown option key instead of
    /// erroring, which is what lets a workbook written against a newer option
    /// set still evaluate.
    pub options: Vec<(String, SparklineValue)>,
}
