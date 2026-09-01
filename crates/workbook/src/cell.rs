use serde::{Deserialize, Deserializer, Serialize};

use crate::error::WorkbookError;
use crate::value::Value;

/// An authored cell: a literal (`value` only) or a formula
/// (`formula` + `value`). Schema spec §4.
///
/// `value` is required on every cell, including formula cells — a
/// never-evaluated formula cell carries [`Value::Empty`] until first recalc.
/// A formula-less cell whose value is empty is invalid; construction and
/// deserialization both reject it (see [`Cell::literal`]).
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(try_from = "CellDe")]
pub struct Cell {
    #[serde(skip_serializing_if = "Option::is_none")]
    formula: Option<String>,
    value: Value,
}

impl Cell {
    /// Creates a literal cell.
    ///
    /// Rejects [`Value::Empty`]: an empty literal would be
    /// byte-distinguishable from the absent cell it denotes (schema spec §4).
    /// Clear a cell by removing its entry from the sheet's cell map instead.
    pub fn literal(value: Value) -> Result<Self, WorkbookError> {
        if matches!(value, Value::Empty) {
            return Err(WorkbookError::EmptyLiteral);
        }
        Ok(Self {
            formula: None,
            value,
        })
    }

    /// Creates a formula cell.
    ///
    /// `formula` is the verbatim authored text including the leading `=` —
    /// it is never normalized and round-trips byte-exact. `value` is the
    /// most recent evaluated result ([`Value::Empty`] until first recalc).
    pub fn with_formula(formula: impl Into<String>, value: Value) -> Self {
        Self {
            formula: Some(formula.into()),
            value,
        }
    }

    /// The most recent evaluated result.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// The verbatim authored formula text, if this is a formula cell.
    pub fn formula(&self) -> Option<&str> {
        self.formula.as_deref()
    }

    /// Replaces the authored formula text, leaving `value` alone.
    ///
    /// Crate-internal, and deliberately not a public setter: the one caller is
    /// [`Workbook::rename_sheet`](crate::Workbook::rename_sheet)'s reference
    /// rewrite, which substitutes a new sheet qualifier into text the engine
    /// re-rendered from the same parse. A public setter would let a caller
    /// swap formula text without invalidating the dependency-graph cache.
    pub(crate) fn set_formula(&mut self, formula: String) {
        self.formula = Some(formula);
    }
}

/// Shadow struct for deserialization: rejects unknown fields (including the
/// reserved `format` and `comment`, schema spec §4 and §9) before the
/// empty-literal invariant is checked in `TryFrom`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CellDe {
    #[serde(default, deserialize_with = "de_formula")]
    formula: Option<String>,
    value: Value,
}

/// `formula`, when present, must be a JSON string (schema spec §4); an
/// explicit `"formula": null` is not schema-valid and is rejected rather
/// than treated as an absent field.
fn de_formula<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}

impl TryFrom<CellDe> for Cell {
    type Error = WorkbookError;

    fn try_from(de: CellDe) -> Result<Self, Self::Error> {
        if de.formula.is_none() && matches!(de.value, Value::Empty) {
            return Err(WorkbookError::EmptyLiteral);
        }
        Ok(Self {
            formula: de.formula,
            value: de.value,
        })
    }
}
