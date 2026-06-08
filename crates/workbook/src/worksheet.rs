use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::cell::Cell;

/// A named sheet holding a sparse cell grid (schema spec §3).
///
/// Cell keys are plain uppercase A1 addresses (`A1`, `BC42`) — no `$`, no
/// sheet qualifier, no lowercase. Absent addresses are empty cells and are
/// never serialized. The map is ordered (`BTreeMap`), which matches the
/// canonical (JCS) key order for ASCII A1 addresses: `A10` sorts before `A2`.
///
/// Address-syntax and sheet-name validation against the schema's normative
/// rules happens at the serialization boundary (`Workbook::from_json`,
/// plan item 2.3), not at construction.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Worksheet {
    cells: BTreeMap<String, Cell>,
    name: String,
}

impl Worksheet {
    /// Creates an empty worksheet named `name`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            cells: BTreeMap::new(),
            name: name.into(),
        }
    }

    /// The sheet name, unique within a workbook (case-insensitive).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The sparse cell grid, keyed by plain uppercase A1 address.
    pub fn cells(&self) -> &BTreeMap<String, Cell> {
        &self.cells
    }

    /// Mutable access to the sparse cell grid.
    pub fn cells_mut(&mut self) -> &mut BTreeMap<String, Cell> {
        &mut self.cells
    }
}
