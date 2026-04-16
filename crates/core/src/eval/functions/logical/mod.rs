use super::super::{FunctionMeta, Registry};

pub mod if_fn;
pub mod and;
pub mod or;
pub mod not;
pub mod iferror;
pub mod ifs;
pub mod switch;
pub mod is_checks;
pub mod constants;
pub mod xor;
pub mod info;
pub mod cell_fn;

pub fn register_logical(registry: &mut Registry) {
    registry.register_lazy("IF",        if_fn::if_fn,               FunctionMeta { category: "logical", signature: "IF(condition, true_val, false_val)",    description: "Conditional value" });
    registry.register_lazy("AND",       and::and_fn,                FunctionMeta { category: "logical", signature: "AND(value1,...)",                       description: "True if all arguments are true" });
    registry.register_lazy("OR",        or::or_fn,                  FunctionMeta { category: "logical", signature: "OR(value1,...)",                        description: "True if any argument is true" });
    registry.register_eager("NOT",      not::not_fn,                FunctionMeta { category: "logical", signature: "NOT(value)",                            description: "Logical negation" });
    registry.register_lazy("IFERROR",   iferror::iferror_fn,        FunctionMeta { category: "logical", signature: "IFERROR(value, value_if_error)",        description: "Return alternate value on error" });
    registry.register_lazy("IFNA",      iferror::ifna_fn,           FunctionMeta { category: "logical", signature: "IFNA(value, value_if_na)",              description: "Return alternate value on #N/A" });
    registry.register_lazy("IFS",       ifs::ifs_fn,                FunctionMeta { category: "logical", signature: "IFS(cond1, val1,...)",                  description: "First value whose condition is true" });
    registry.register_lazy("SWITCH",    switch::switch_fn,          FunctionMeta { category: "logical", signature: "SWITCH(expr, val1, result1,...)",       description: "Match expression against values" });
    registry.register_lazy("ISNUMBER",  is_checks::isnumber_lazy_fn,FunctionMeta { category: "logical", signature: "ISNUMBER(value)",                      description: "True if value is a number" });
    registry.register_lazy("ISTEXT",    is_checks::istext_lazy_fn,  FunctionMeta { category: "logical", signature: "ISTEXT(value)",                        description: "True if value is text" });
    registry.register_lazy("ISERROR",   is_checks::iserror_lazy_fn, FunctionMeta { category: "logical", signature: "ISERROR(value)",                       description: "True if value is an error" });
    registry.register_lazy("ISBLANK",   is_checks::isblank_lazy_fn, FunctionMeta { category: "logical", signature: "ISBLANK(value)",                       description: "True if value is blank" });
    registry.register_lazy("ISNA",      is_checks::isna_lazy_fn,    FunctionMeta { category: "logical", signature: "ISNA(value)",                          description: "True if value is #N/A" });
    registry.register_lazy("ISERR",     is_checks::iserr_fn,        FunctionMeta { category: "logical", signature: "ISERR(value)",                         description: "True if value is an error other than #N/A" });
    registry.register_lazy("ISLOGICAL", is_checks::islogical_fn,    FunctionMeta { category: "logical", signature: "ISLOGICAL(value)",                     description: "True if value is a logical (boolean)" });
    registry.register_lazy("ISNONTEXT", is_checks::isnontext_fn,    FunctionMeta { category: "logical", signature: "ISNONTEXT(value)",                     description: "True if value is not text" });
    registry.register_eager("NA",       constants::na_fn,           FunctionMeta { category: "logical", signature: "NA()",                                 description: "Returns the #N/A error value" });
    registry.register_eager("TRUE",     constants::true_fn,         FunctionMeta { category: "logical", signature: "TRUE()",                               description: "Logical true value" });
    registry.register_eager("FALSE",    constants::false_fn,        FunctionMeta { category: "logical", signature: "FALSE()",                              description: "Logical false value" });
    registry.register_lazy("XOR",       xor::xor_fn,                FunctionMeta { category: "logical", signature: "XOR(value1,...)",                      description: "True if an odd number of arguments are true" });
    registry.register_lazy("ERROR.TYPE",info::error_type_fn,        FunctionMeta { category: "logical", signature: "ERROR.TYPE(value)",                    description: "Number corresponding to the error type" });
    registry.register_lazy("N",         info::n_fn,                 FunctionMeta { category: "logical", signature: "N(value)",                             description: "Convert value to number" });
    registry.register_lazy("TYPE",      info::type_fn,              FunctionMeta { category: "logical", signature: "TYPE(value)",                          description: "Number indicating value type" });
    registry.register_lazy("ISREF",     is_checks::isref_fn,        FunctionMeta { category: "logical", signature: "ISREF(value)",                         description: "True if value is a cell reference" });
    registry.register_lazy("ISFORMULA", is_checks::isformula_fn,    FunctionMeta { category: "logical", signature: "ISFORMULA(ref)",                       description: "True if cell contains a formula" });
    registry.register_lazy("CELL",      cell_fn::cell_fn,           FunctionMeta { category: "logical", signature: "CELL(info_type, reference)",             description: "Returns information about a cell" });
}
