use super::super::Registry;

pub mod if_fn;

pub fn register_logical(registry: &mut Registry) {
    registry.register_lazy("IF", if_fn::if_fn);
}
