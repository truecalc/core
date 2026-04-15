use super::super::Registry;

pub mod sum;

pub fn register_math(registry: &mut Registry) {
    registry.register_eager("SUM", sum::sum_fn);
}
