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
        }
    }
}

impl std::error::Error for WorkbookError {}
