use super::{FunctionMeta, Registry};

pub mod indirect;

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
}
