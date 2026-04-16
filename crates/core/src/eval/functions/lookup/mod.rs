use super::Registry;
use super::FunctionMeta;

pub mod index_fn;

pub fn register_lookup(registry: &mut Registry) {
    registry.register_eager("INDEX", index_fn::index_fn, FunctionMeta {
        category: "lookup",
        signature: "INDEX(array, row, [col])",
        description: "Return the element at the given position in an array",
    });
}
