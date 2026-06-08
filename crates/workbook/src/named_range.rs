use serde::{Deserialize, Serialize};

/// A workbook-scoped named range (schema spec §7).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedRange {
    /// The name, e.g. `TaxRate`. Unique across the workbook
    /// (case-insensitive); must not parse as an A1 address, an R1C1-style
    /// reference, or a boolean literal.
    pub name: String,
    /// Canonical sheet-qualified A1 reference: `Sheet!A1` or `Sheet!A1:B2`.
    #[serde(rename = "ref")]
    pub r#ref: String,
}
