pub mod convert;
pub mod to_date;
pub mod to_dollars;
pub mod to_percent;
pub mod to_pure_number;
pub mod to_text;

use super::{FunctionMeta, Registry};

pub fn register_parser(registry: &mut Registry) {
    registry.register_eager("CONVERT",        convert::convert_fn,               FunctionMeta { category: "parser", signature: "CONVERT(value,from_unit,to_unit)", description: "Converts a number from one unit of measurement to another" });
    registry.register_eager("TO_DATE",        to_date::to_date_fn,             FunctionMeta { category: "parser", signature: "TO_DATE(value)",        description: "Converts a number to a date serial value" });
    registry.register_eager("TO_DOLLARS",     to_dollars::to_dollars_fn,       FunctionMeta { category: "parser", signature: "TO_DOLLARS(value)",     description: "Formats a number as a dollar amount" });
    registry.register_eager("TO_PERCENT",     to_percent::to_percent_fn,       FunctionMeta { category: "parser", signature: "TO_PERCENT(value)",     description: "Formats a number as a percentage" });
    registry.register_eager("TO_PURE_NUMBER", to_pure_number::to_pure_number_fn, FunctionMeta { category: "parser", signature: "TO_PURE_NUMBER(value)", description: "Strips formatting and returns a plain number" });
    registry.register_eager("TO_TEXT",        to_text::to_text_fn,             FunctionMeta { category: "parser", signature: "TO_TEXT(value)",        description: "Converts a value to its text representation" });
}
