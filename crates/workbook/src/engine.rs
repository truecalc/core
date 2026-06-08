use serde::{Deserialize, Serialize};

/// Which spreadsheet product's semantics every formula in a workbook targets.
///
/// Engine flavor is explicit and required on every public entry point; there
/// is no default (ADR 2026-04-27-engine-flavor-explicit-everywhere).
/// Serialized as the workbook's required top-level `engine` field:
/// `"sheets"` | `"excel"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineFlavor {
    /// Google Sheets semantics (day 0 of the date serial system is 1899-12-30).
    Sheets,
    /// Excel semantics (1900 date system, including the historical
    /// 1900 leap-year bug).
    Excel,
}
