use std::fmt;

/// Errors from workbook value-type constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorkbookError {
    /// A formula-less cell whose value is `empty` is invalid: it would be
    /// byte-distinguishable from the absent cell it denotes, breaking
    /// canonical uniqueness (schema spec §4). Clear a cell by removing its
    /// entry from the sheet's cell map instead.
    EmptyLiteral,
}

impl fmt::Display for WorkbookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkbookError::EmptyLiteral => write!(
                f,
                "a literal cell cannot hold an empty value; \
                 clear a cell by removing its entry instead"
            ),
        }
    }
}

impl std::error::Error for WorkbookError {}
