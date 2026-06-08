use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::canonical;
use crate::engine::EngineFlavor;
use crate::error::WorkbookError;
use crate::limits;
use crate::named_range::NamedRange;
use crate::strict_json;
use crate::validate;
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

    /// Parses a workbook from JSON bytes, enforcing every document-level rule
    /// of the schema (schema spec §1–§10) and the resource limits of the scope
    /// ADR (Decision 5).
    ///
    /// Accepts any schema-valid JSON — pretty-printed, reordered keys, extra
    /// whitespace are all fine; only the *content* must be valid (schema spec
    /// §8: non-canonical-but-valid input is accepted, output is always
    /// canonical). Beyond the serde layer's checks (unknown fields, value
    /// encodings incl. NaN/Inf and `-0`, empty-literal, exact version match),
    /// this enforces the rules serde cannot express:
    ///
    /// - **§1** duplicate object keys are rejected; a UTF-8 BOM and invalid
    ///   UTF-8 are rejected at the byte boundary (hence `&[u8]`, not `&str`);
    /// - **§2/§3** sheet names are non-empty, ≤ 100 scalar values, and unique
    ///   under Unicode **simple** case folding;
    /// - **§3** cell keys match `^[A-Z]{1,3}[1-9][0-9]{0,7}$` and lie within
    ///   the address bounds;
    /// - **§5** spill rectangles are document-valid (no authored cell inside an
    ///   anchor's rectangle, no overlapping rectangles, none out of bounds);
    /// - **§7** named-range names and `ref`s are valid and canonical, names are
    ///   unique case-insensitively, and no `ref` dangles to a missing sheet;
    /// - **Decision 5** input size and all structural limits are enforced.
    pub fn from_json(bytes: &[u8]) -> Result<Self, WorkbookError> {
        if bytes.len() > limits::MAX_SERIALIZED_BYTES {
            return Err(WorkbookError::Validation(format!(
                "input is {} bytes, exceeding the {}-byte limit (scope ADR Decision 5)",
                bytes.len(),
                limits::MAX_SERIALIZED_BYTES
            )));
        }
        // §1: duplicate-key- and BOM-rejecting parse into a JSON tree.
        let tree = strict_json::parse_no_dup_keys(bytes).map_err(WorkbookError::Validation)?;
        // Document-level invariants serde cannot express (§2/§3/§5/§7, limits).
        validate::validate_document(&tree).map_err(WorkbookError::Validation)?;
        // Typed deserialization (unknown fields, value encodings, version,
        // empty-literal) — the serde layer of P2.2.
        serde_json::from_value(tree).map_err(|e| WorkbookError::Validation(e.to_string()))
    }

    /// Serializes the workbook to its canonical RFC 8785 (JCS) byte form
    /// (schema spec §8): one line, no insignificant whitespace, no trailing
    /// newline, object keys sorted by UTF-16 code units, ECMAScript number
    /// formatting, `names` sorted by `name`.
    ///
    /// Errors if a value is non-finite (forbidden, schema spec §8.4) or if the
    /// canonical bytes exceed the 100 MiB cap (scope ADR Decision 5).
    pub fn to_json(&self) -> Result<String, WorkbookError> {
        // Serialize through the typed serde layer (which already emits the §6
        // value encodings and rejects NaN/Inf), then canonicalize the tree.
        let mut tree =
            serde_json::to_value(self).map_err(|e| WorkbookError::Validation(e.to_string()))?;
        sort_names_by_name(&mut tree);
        let canonical = canonical::to_canonical_string(&tree).map_err(WorkbookError::Validation)?;
        if canonical.len() > limits::MAX_SERIALIZED_BYTES {
            return Err(WorkbookError::Validation(format!(
                "canonical workbook is {} bytes, exceeding the {}-byte limit                  (scope ADR Decision 5)",
                canonical.len(),
                limits::MAX_SERIALIZED_BYTES
            )));
        }
        Ok(canonical)
    }
}

/// Domain ordering of schema spec §8.7: `names` is serialized sorted by `name`
/// in ascending UTF-16 code-unit order (matching JCS string ordering).
/// `sheets` keeps authored tab order (it is data, not a set) and is left
/// untouched.
fn sort_names_by_name(tree: &mut serde_json::Value) {
    if let Some(names) = tree.get_mut("names").and_then(|v| v.as_array_mut()) {
        names.sort_by(|a, b| {
            let an = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let bn = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
            an.encode_utf16().cmp(bn.encode_utf16())
        });
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
