use super::super::Registry;

pub mod if_fn;
pub mod and;
pub mod or;
pub mod not;
pub mod iferror;
pub mod ifs;
pub mod switch;
pub mod is_checks;

pub fn register_logical(registry: &mut Registry) {
    registry.register_lazy("IF", if_fn::if_fn);
    registry.register_lazy("AND", and::and_fn);
    registry.register_lazy("OR", or::or_fn);
    registry.register_eager("NOT", not::not_fn);
    registry.register_lazy("IFERROR", iferror::iferror_fn);
    registry.register_lazy("IFNA", iferror::ifna_fn);
    registry.register_lazy("IFS", ifs::ifs_fn);
    registry.register_lazy("SWITCH", switch::switch_fn);
    registry.register_eager("ISNUMBER", is_checks::isnumber_fn);
    registry.register_eager("ISTEXT", is_checks::istext_fn);
    registry.register_eager("ISERROR", is_checks::iserror_fn);
    registry.register_eager("ISBLANK", is_checks::isblank_fn);
    registry.register_eager("ISNA", is_checks::isna_fn);
}
