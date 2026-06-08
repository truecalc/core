use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::address::Address;
use crate::cell::Cell;

/// A named sheet holding a sparse cell grid (schema spec §3).
///
/// Cell keys are plain uppercase A1 addresses (`A1`, `BC42`) — no `$`, no
/// sheet qualifier, no lowercase. Absent addresses are empty cells and are
/// never serialized. The map is ordered (`BTreeMap`), which matches the
/// canonical (JCS) key order for ASCII A1 addresses: `A10` sorts before `A2`.
///
/// The typed grid API ([`get`](Self::get) / [`set`](Self::set) /
/// [`clear`](Self::clear), keyed by a bounds-validated [`Address`]) is the
/// in-memory accessor used by the recalc runtime (plan item 3.1); the raw
/// [`cells`](Self::cells) map is retained for the serde layer. Address-syntax
/// and sheet-name validation against the schema's normative rules also happens
/// at the serialization boundary ([`Workbook::from_json`](crate::Workbook::from_json)),
/// independently of the typed API, since `from_json` parses untrusted keys.
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

    /// Sets the sheet name. Crate-internal: uniqueness and length validation is
    /// the workbook's job ([`Workbook::rename_sheet`](crate::Workbook::rename_sheet)),
    /// the only public path to a rename.
    pub(crate) fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// The sparse cell grid, keyed by plain uppercase A1 address.
    pub fn cells(&self) -> &BTreeMap<String, Cell> {
        &self.cells
    }

    /// Mutable access to the sparse cell grid.
    pub fn cells_mut(&mut self) -> &mut BTreeMap<String, Cell> {
        &mut self.cells
    }

    /// The cell at `addr`, or `None` if that address is empty.
    pub fn get(&self, addr: Address) -> Option<&Cell> {
        self.cells.get(&addr.to_a1())
    }

    /// Mutable access to the cell at `addr`, or `None` if that address is empty.
    pub fn get_mut(&mut self, addr: Address) -> Option<&mut Cell> {
        self.cells.get_mut(&addr.to_a1())
    }

    /// Writes `cell` at `addr`, returning the cell previously there (if any).
    ///
    /// `addr` is already bounds-validated by construction, so a set never
    /// escapes the address bounds. To clear a cell use [`clear`](Self::clear),
    /// never a set of an empty value (schema spec §4).
    pub fn set(&mut self, addr: Address, cell: Cell) -> Option<Cell> {
        self.cells.insert(addr.to_a1(), cell)
    }

    /// Removes the cell at `addr`, returning it if present.
    ///
    /// Clearing a cell is *removing its entry*, never writing an empty value:
    /// an empty entry would be byte-distinguishable from the absent cell it
    /// denotes (schema spec §4).
    pub fn clear(&mut self, addr: Address) -> Option<Cell> {
        self.cells.remove(&addr.to_a1())
    }

    /// Whether a cell is present at `addr`.
    pub fn contains(&self, addr: Address) -> bool {
        self.cells.contains_key(&addr.to_a1())
    }

    /// The number of populated cells.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Whether the grid has no populated cells.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Iterates the populated cells in canonical (JCS) key order, yielding the
    /// parsed [`Address`] for each. Keys held by a worksheet are always valid
    /// A1, so parsing cannot fail.
    pub fn iter(&self) -> impl Iterator<Item = (Address, &Cell)> {
        self.cells.iter().map(|(key, cell)| {
            let addr = Address::from_a1(key).expect("worksheet keys are always valid A1");
            (addr, cell)
        })
    }
}
