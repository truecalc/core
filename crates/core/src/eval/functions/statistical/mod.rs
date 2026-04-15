use super::super::Registry;

pub mod count;
pub mod max;
pub mod median;
pub mod min;

pub fn register_statistical(registry: &mut Registry) {
    registry.register_lazy("COUNT",  count::count_lazy_fn);
    registry.register_lazy("COUNTA", count::counta_lazy_fn);
    registry.register_eager("MAX",    max::max_fn);
    registry.register_eager("MIN",    min::min_fn);
    registry.register_eager("MEDIAN", median::median_fn);
}
