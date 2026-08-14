use serde::{Deserialize, Serialize};

/// A workbook-scoped table declaration (structured-references spec §4).
/// Mirrors [`crate::named_range::NamedRange`]'s shape exactly: `{name, ref}`.
/// The range's first row is always the header row in v1 — no separate
/// `headerRow` field (nothing needs the header anywhere else yet).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    /// The table's name, e.g. `Recipe`. Unique across the workbook among
    /// both table names and named-range names (case-insensitive).
    pub name: String,
    /// Canonical sheet-qualified A1 range: `Sheet!A1:D12`. Always a range
    /// (never collapses to a single-cell form, even for a one-data-row table).
    #[serde(rename = "ref")]
    pub r#ref: String,
}

#[cfg(test)]
mod tests;
