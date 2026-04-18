use super::{FunctionMeta, Registry};

pub mod address;
pub mod array_utils;
pub mod cell_ref;
pub mod choose;
pub mod index_match;
pub mod indirect;
pub mod lookup_fn;
pub mod misc;
pub mod row_col;
pub mod vlookup;

#[cfg(test)]
mod tests;

pub fn register_lookup(registry: &mut Registry) {
    registry.register_eager(
        "INDIRECT",
        indirect::indirect_fn,
        FunctionMeta {
            category: "lookup",
            signature: "INDIRECT(ref_text, [a1])",
            description: "Returns the value of a cell reference given as a string",
        },
    );
    registry.register_eager(
        "ADDRESS",
        address::address_fn,
        FunctionMeta {
            category: "lookup",
            signature: "ADDRESS(row, col, [abs_mode], [a1], [sheet_text])",
            description: "Returns a cell address string",
        },
    );
    registry.register_lazy(
        "CHOOSE",
        choose::choose_fn,
        FunctionMeta {
            category: "lookup",
            signature: "CHOOSE(index, value1, value2, ...)",
            description: "Returns the value at the given 1-based index",
        },
    );
    registry.register_lazy(
        "ROW",
        row_col::row_fn,
        FunctionMeta {
            category: "lookup",
            signature: "ROW([cell_ref])",
            description: "Returns the row number of a cell reference",
        },
    );
    registry.register_lazy(
        "COLUMN",
        row_col::column_fn,
        FunctionMeta {
            category: "lookup",
            signature: "COLUMN([cell_ref])",
            description: "Returns the column number of a cell reference",
        },
    );
    registry.register_lazy(
        "ROWS",
        row_col::rows_fn,
        FunctionMeta {
            category: "lookup",
            signature: "ROWS(array_or_range)",
            description: "Returns the number of rows in an array or range",
        },
    );
    registry.register_lazy(
        "COLUMNS",
        row_col::columns_fn,
        FunctionMeta {
            category: "lookup",
            signature: "COLUMNS(array_or_range)",
            description: "Returns the number of columns in an array or range",
        },
    );
    registry.register_eager(
        "VLOOKUP",
        vlookup::vlookup_fn,
        FunctionMeta {
            category: "lookup",
            signature: "VLOOKUP(search_key, range, index, [is_sorted])",
            description: "Searches first column of range, returns value from index column",
        },
    );
    registry.register_eager(
        "HLOOKUP",
        vlookup::hlookup_fn,
        FunctionMeta {
            category: "lookup",
            signature: "HLOOKUP(search_key, range, index, [is_sorted])",
            description: "Searches first row of range, returns value from index row",
        },
    );
    registry.register_eager(
        "MATCH",
        index_match::match_fn,
        FunctionMeta {
            category: "lookup",
            signature: "MATCH(search_key, range, [match_type])",
            description: "Returns 1-based position of first match",
        },
    );
    registry.register_eager(
        "LOOKUP",
        lookup_fn::lookup_fn,
        FunctionMeta {
            category: "lookup",
            signature: "LOOKUP(search_key, search_range, [result_range])",
            description: "Approximate lookup in sorted range",
        },
    );
    registry.register_eager(
        "XLOOKUP",
        lookup_fn::xlookup_fn,
        FunctionMeta {
            category: "lookup",
            signature: "XLOOKUP(search_key, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])",
            description: "Modern lookup function with fallback and match options",
        },
    );
    registry.register_eager(
        "XMATCH",
        lookup_fn::xmatch_fn,
        FunctionMeta {
            category: "lookup",
            signature: "XMATCH(search_key, lookup_array, [match_mode], [search_mode])",
            description: "Modern MATCH function with match and search mode options",
        },
    );
    registry.register_lazy(
        "FORMULATEXT",
        misc::formulatext_fn,
        FunctionMeta {
            category: "lookup",
            signature: "FORMULATEXT(reference)",
            description: "Returns the formula string of a cell reference",
        },
    );
    registry.register_lazy(
        "GETPIVOTDATA",
        misc::getpivotdata_fn,
        FunctionMeta {
            category: "lookup",
            signature: "GETPIVOTDATA(value_name, pivot_table, ...)",
            description: "Returns data from a pivot table",
        },
    );
    registry.register_lazy(
        "OFFSET",
        misc::offset_fn,
        FunctionMeta {
            category: "lookup",
            signature: "OFFSET(reference, rows, cols, [height], [width])",
            description: "Returns a reference offset from a base reference",
        },
    );
    registry.register_lazy(
        "SHEET",
        misc::sheet_fn,
        FunctionMeta {
            category: "lookup",
            signature: "SHEET([name])",
            description: "Returns the sheet number of the current or named sheet",
        },
    );
}
