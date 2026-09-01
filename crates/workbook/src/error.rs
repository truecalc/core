use std::fmt;

/// Errors from workbook value-type constructors and from
/// [`Workbook::from_json`](crate::Workbook::from_json) /
/// [`Workbook::to_json`](crate::Workbook::to_json).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorkbookError {
    /// A formula-less cell whose value is `empty` is invalid: it would be
    /// byte-distinguishable from the absent cell it denotes, breaking
    /// canonical uniqueness (schema spec §4). Clear a cell by removing its
    /// entry from the sheet's cell map instead.
    EmptyLiteral,
    /// A document-level rule was violated while parsing or serializing — a
    /// rule serde cannot express (schema spec §1, §2, §3, §5, §7, §8) or a
    /// resource-limit breach (scope ADR Decision 5). Carries a human-readable
    /// description of the first violation found.
    Validation(String),
    /// A sheet-management operation
    /// ([`add_sheet`](crate::Workbook::add_sheet),
    /// [`insert_sheet`](crate::Workbook::insert_sheet),
    /// [`rename_sheet`](crate::Workbook::rename_sheet),
    /// [`move_sheet`](crate::Workbook::move_sheet)) violated an invariant —
    /// a duplicate sheet name under simple case folding (schema spec §2), a
    /// name that is empty or exceeds the length limit (schema spec §3), the
    /// per-workbook sheet cap (scope ADR Decision 5), an out-of-range tab
    /// position, or an unknown sheet name. Carries a human-readable
    /// description.
    SheetManagement(String),
    /// [`Workbook::set`](crate::Workbook::set) directly named a sheet, by
    /// name, that does not exist. Distinct from [`DanglingSheetRef`] — this
    /// is the direct-operation-target case (a caller-supplied sheet name
    /// argument), not an indirect reference embedded inside a
    /// named-range/table `ref` string. Carries a human-readable description.
    ///
    /// [`DanglingSheetRef`]: WorkbookError::DanglingSheetRef
    UnknownSheet(String),
    /// A named-range or table `ref` string parses fine but names a sheet the
    /// workbook does not have (schema spec §7's dangling-ref rule, checked
    /// eagerly at definition time rather than only at save). Distinct from
    /// [`UnknownSheet`] — the caller here supplied a `ref` *string* to
    /// validate, not a bare sheet-name argument. Carries a human-readable
    /// description.
    ///
    /// [`UnknownSheet`]: WorkbookError::UnknownSheet
    DanglingSheetRef(String),
    /// A formula string failed to parse against the workbook's locked engine
    /// ([`Workbook::set`](crate::Workbook::set)'s syntax-only validation).
    /// Carries a human-readable description.
    InvalidFormula(String),
    /// A named-range/table *name* or *ref* is syntactically invalid or
    /// non-canonical, independent of sheet existence: an invalid name shape
    /// (must match `^[A-Za-z_][A-Za-z0-9_]*$` and not be an A1/R1C1/boolean
    /// literal), a `ref` that fails to parse as a canonical reference, or a
    /// `ref` that parses but is not a range where a range is required
    /// (tables only). Carries a human-readable description.
    InvalidReference(String),
    /// A new named range or table's name collides, case-insensitively, with
    /// an existing named range or table (cross-kind collisions included,
    /// since names and tables share one case-insensitive namespace per
    /// structured-references spec §4). Carries a human-readable description.
    DuplicateName(String),
    /// A table's range overlaps another table's range
    /// (structured-references spec §4 — table ranges must be disjoint).
    /// Distinct from [`DuplicateName`] — the name is fine, the geometry
    /// conflicts. Carries a human-readable description.
    ///
    /// [`DuplicateName`]: WorkbookError::DuplicateName
    RangeOverlap(String),
    /// A redefine targeted a name/table that does not currently exist (the
    /// update-only counterpart of [`DuplicateName`]'s create-time collision
    /// check). Carries a human-readable description.
    ///
    /// [`DuplicateName`]: WorkbookError::DuplicateName
    NotFound(String),
    /// A per-mutation resource cap from scope ADR Decision 5 was exceeded:
    /// workbook cell count, named-range count, table count, a text value's
    /// scalar-value length, an array value's element count, or a formula's
    /// byte length. Carries a human-readable description.
    ResourceLimit(String),
}

impl fmt::Display for WorkbookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkbookError::EmptyLiteral => write!(
                f,
                "a literal cell cannot hold an empty value; \
                 clear a cell by removing its entry instead"
            ),
            WorkbookError::Validation(msg) => write!(f, "{msg}"),
            WorkbookError::SheetManagement(msg) => write!(f, "{msg}"),
            WorkbookError::UnknownSheet(msg) => write!(f, "{msg}"),
            WorkbookError::DanglingSheetRef(msg) => write!(f, "{msg}"),
            WorkbookError::InvalidFormula(msg) => write!(f, "{msg}"),
            WorkbookError::InvalidReference(msg) => write!(f, "{msg}"),
            WorkbookError::DuplicateName(msg) => write!(f, "{msg}"),
            WorkbookError::RangeOverlap(msg) => write!(f, "{msg}"),
            WorkbookError::NotFound(msg) => write!(f, "{msg}"),
            WorkbookError::ResourceLimit(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for WorkbookError {}
