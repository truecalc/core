use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::engine::EngineFlavor;
use crate::named_range::NamedRange;
use crate::worksheet::Worksheet;

/// The schema version this library writes (schema spec §10). A string, not
/// an integer: compared by exact match, never numerically.
pub const SCHEMA_VERSION: &str = "1";

/// An engine-locked spreadsheet workbook — a pure value object (no hidden
/// state, no callbacks). Schema spec §2.
///
/// All four fields are always serialized, even when empty. Field declaration
/// order (`engine`, `names`, `sheets`, `version`) matches canonical (JCS)
/// key order.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workbook {
    engine: EngineFlavor,
    names: Vec<NamedRange>,
    sheets: Vec<Worksheet>,
    #[serde(deserialize_with = "de_version")]
    version: String,
}

impl Workbook {
    /// Creates an empty workbook locked to `engine`.
    ///
    /// The engine flavor is required at creation and immutable for the
    /// workbook's lifetime (ADR 2026-04-27-engine-flavor-explicit-everywhere):
    /// there is no default and no setter.
    pub fn new(engine: EngineFlavor) -> Self {
        Self {
            engine,
            names: Vec::new(),
            sheets: Vec::new(),
            version: SCHEMA_VERSION.to_owned(),
        }
    }

    /// The engine flavor every formula in this workbook targets.
    pub fn engine(&self) -> EngineFlavor {
        self.engine
    }

    /// The schema version of this workbook document.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The worksheets, in tab order (array position is tab position).
    pub fn sheets(&self) -> &[Worksheet] {
        &self.sheets
    }

    /// Mutable access to the worksheets.
    pub fn sheets_mut(&mut self) -> &mut Vec<Worksheet> {
        &mut self.sheets
    }

    /// The workbook-scoped named ranges.
    pub fn names(&self) -> &[NamedRange] {
        &self.names
    }

    /// Mutable access to the named ranges.
    pub fn names_mut(&mut self) -> &mut Vec<NamedRange> {
        &mut self.names
    }
}

/// Reader rule of schema spec §10: accept every version this library
/// knows (`"1"`), reject unknown versions with a clear "upgrade" error.
fn de_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != SCHEMA_VERSION {
        return Err(D::Error::custom(format!(
            "unsupported schema version {version:?}: this version of \
             truecalc-workbook reads version \"1\" (schema spec §10); \
             upgrade truecalc to load this workbook"
        )));
    }
    Ok(version)
}
