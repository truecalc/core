use super::super::{FunctionMeta, Registry};

pub mod count;
pub mod countblank;
pub mod max;
pub mod median;
pub mod min;

pub fn register_statistical(registry: &mut Registry) {
    registry.register_lazy("COUNT",  count::count_lazy_fn,  FunctionMeta { category: "statistical", signature: "COUNT(value1,...)",  description: "Count numeric values" });
    registry.register_lazy("COUNTA", count::counta_lazy_fn, FunctionMeta { category: "statistical", signature: "COUNTA(value1,...)", description: "Count non-empty values" });
    registry.register_eager("COUNTBLANK", countblank::countblank_fn, FunctionMeta { category: "statistical", signature: "COUNTBLANK(range)", description: "Count blank/empty cells" });
    registry.register_eager("MAX",   max::max_fn,           FunctionMeta { category: "statistical", signature: "MAX(value1,...)",    description: "Maximum value" });
    registry.register_eager("MIN",   min::min_fn,           FunctionMeta { category: "statistical", signature: "MIN(value1,...)",    description: "Minimum value" });
    registry.register_eager("MEDIAN",median::median_fn,     FunctionMeta { category: "statistical", signature: "MEDIAN(value1,...)", description: "Median value" });
}
