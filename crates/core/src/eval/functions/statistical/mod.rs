use super::super::{FunctionMeta, Registry};

pub mod count;
pub mod countblank;
pub mod geomean;
pub mod harmean;
pub mod kurt;
pub mod max;
pub mod median;
pub mod min;
pub mod mode;
pub mod mode_mult;
pub mod mode_sngl;
pub mod skew;
pub mod skew_p;
pub mod stat_helpers;
pub mod trimmean;

pub mod avedev;
pub mod covariance_p;
pub mod covariance_s;
pub mod devsq;
pub mod stdev;
pub mod stdev_p;
pub mod stdev_s;
pub mod stdeva;
pub mod stdevp;
pub mod stdevpa;
pub mod var;
pub mod var_p;
pub mod var_s;
pub mod vara;
pub mod varp;
pub mod varpa;

pub fn register_statistical(registry: &mut Registry) {
    registry.register_lazy("COUNT",  count::count_lazy_fn,  FunctionMeta { category: "statistical", signature: "COUNT(value1,...)",  description: "Count numeric values" });
    registry.register_lazy("COUNTA", count::counta_lazy_fn, FunctionMeta { category: "statistical", signature: "COUNTA(value1,...)", description: "Count non-empty values" });
    registry.register_eager("COUNTBLANK", countblank::countblank_fn, FunctionMeta { category: "statistical", signature: "COUNTBLANK(range)", description: "Count blank/empty cells" });
    registry.register_eager("GEOMEAN",  geomean::geomean_fn,  FunctionMeta { category: "statistical", signature: "GEOMEAN(value1,...)", description: "Geometric mean" });
    registry.register_eager("HARMEAN",  harmean::harmean_fn,  FunctionMeta { category: "statistical", signature: "HARMEAN(value1,...)", description: "Harmonic mean" });
    registry.register_eager("KURT",     kurt::kurt_fn,        FunctionMeta { category: "statistical", signature: "KURT(value1,...)",    description: "Excess kurtosis" });
    registry.register_eager("MAX",   max::max_fn,           FunctionMeta { category: "statistical", signature: "MAX(value1,...)",    description: "Maximum value" });
    registry.register_eager("MEDIAN",median::median_fn,     FunctionMeta { category: "statistical", signature: "MEDIAN(value1,...)", description: "Median value" });
    registry.register_eager("MIN",   min::min_fn,           FunctionMeta { category: "statistical", signature: "MIN(value1,...)",    description: "Minimum value" });
    registry.register_eager("MODE",      mode::mode_fn,           FunctionMeta { category: "statistical", signature: "MODE(value1,...)",      description: "Most frequent value" });
    registry.register_eager("MODE.MULT", mode_mult::mode_mult_fn, FunctionMeta { category: "statistical", signature: "MODE.MULT(value1,...)", description: "All most frequent values" });
    registry.register_eager("MODE.SNGL", mode_sngl::mode_sngl_fn, FunctionMeta { category: "statistical", signature: "MODE.SNGL(value1,...)", description: "Most frequent value" });
    registry.register_eager("SKEW",   skew::skew_fn,        FunctionMeta { category: "statistical", signature: "SKEW(value1,...)",   description: "Sample skewness" });
    registry.register_eager("SKEW.P", skew_p::skew_p_fn,   FunctionMeta { category: "statistical", signature: "SKEW.P(value1,...)", description: "Population skewness" });
    registry.register_eager("TRIMMEAN", trimmean::trimmean_fn, FunctionMeta { category: "statistical", signature: "TRIMMEAN(data,percent)", description: "Trimmed mean" });

    registry.register_eager("AVEDEV",      avedev::avedev_fn,           FunctionMeta { category: "statistical", signature: "AVEDEV(value1,...)",       description: "Average of absolute deviations from the mean" });
    registry.register_eager("DEVSQ",       devsq::devsq_fn,             FunctionMeta { category: "statistical", signature: "DEVSQ(value1,...)",        description: "Sum of squared deviations from the mean" });
    registry.register_eager("COVARIANCE.P",covariance_p::covariance_p_fn, FunctionMeta { category: "statistical", signature: "COVARIANCE.P(array1,array2)", description: "Population covariance" });
    registry.register_eager("COVARIANCE.S",covariance_s::covariance_s_fn, FunctionMeta { category: "statistical", signature: "COVARIANCE.S(array1,array2)", description: "Sample covariance" });
    registry.register_eager("STDEV",       stdev::stdev_fn,             FunctionMeta { category: "statistical", signature: "STDEV(value1,...)",         description: "Sample standard deviation" });
    registry.register_eager("STDEV.P",     stdev_p::stdev_p_fn,         FunctionMeta { category: "statistical", signature: "STDEV.P(value1,...)",       description: "Population standard deviation" });
    registry.register_eager("STDEV.S",     stdev_s::stdev_s_fn,         FunctionMeta { category: "statistical", signature: "STDEV.S(value1,...)",       description: "Sample standard deviation" });
    registry.register_eager("STDEVA",      stdeva::stdeva_fn,           FunctionMeta { category: "statistical", signature: "STDEVA(value1,...)",        description: "Sample standard deviation including text and logical values" });
    registry.register_eager("STDEVP",      stdevp::stdevp_fn,           FunctionMeta { category: "statistical", signature: "STDEVP(value1,...)",        description: "Population standard deviation" });
    registry.register_eager("STDEVPA",     stdevpa::stdevpa_fn,         FunctionMeta { category: "statistical", signature: "STDEVPA(value1,...)",       description: "Population standard deviation including text and logical values" });
    registry.register_eager("VAR",         var::var_fn,                 FunctionMeta { category: "statistical", signature: "VAR(value1,...)",           description: "Sample variance" });
    registry.register_eager("VAR.P",       var_p::var_p_fn,             FunctionMeta { category: "statistical", signature: "VAR.P(value1,...)",         description: "Population variance" });
    registry.register_eager("VAR.S",       var_s::var_s_fn,             FunctionMeta { category: "statistical", signature: "VAR.S(value1,...)",         description: "Sample variance" });
    registry.register_eager("VARA",        vara::vara_fn,               FunctionMeta { category: "statistical", signature: "VARA(value1,...)",          description: "Sample variance including text and logical values" });
    registry.register_eager("VARP",        varp::varp_fn,               FunctionMeta { category: "statistical", signature: "VARP(value1,...)",          description: "Population variance" });
    registry.register_eager("VARPA",       varpa::varpa_fn,             FunctionMeta { category: "statistical", signature: "VARPA(value1,...)",         description: "Population variance including text and logical values" });
}
