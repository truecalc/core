use super::Registry;

pub mod char_fn;
pub mod code_fn;
pub mod concatenate;
pub mod exact;
pub mod find;
pub mod left;
pub mod len;
pub mod lower;
pub mod mid;
pub mod replace;
pub mod rept;
pub mod right;
pub mod substitute;
pub mod t_fn;
pub mod text_fn;
pub mod trim;
pub mod upper;
pub mod value_fn;

pub fn register_text(registry: &mut Registry) {
    registry.register_eager("LEFT",        left::left_fn);
    registry.register_eager("MID",         mid::mid_fn);
    registry.register_eager("RIGHT",       right::right_fn);
    registry.register_eager("LEN",         len::len_fn);
    registry.register_eager("LOWER",       lower::lower_fn);
    registry.register_eager("UPPER",       upper::upper_fn);
    registry.register_eager("TRIM",        trim::trim_fn);
    registry.register_eager("CONCATENATE", concatenate::concatenate_fn);
    registry.register_eager("FIND",        find::find_fn);
    registry.register_eager("SUBSTITUTE",  substitute::substitute_fn);
    registry.register_eager("REPLACE",     replace::replace_fn);
    registry.register_eager("TEXT",        text_fn::text_fn);
    registry.register_eager("VALUE",       value_fn::value_fn);
    registry.register_eager("REPT",        rept::rept_fn);
    registry.register_eager("CHAR",        char_fn::char_fn);
    registry.register_eager("UNICHAR",     char_fn::unichar_fn);
    registry.register_eager("CODE",        code_fn::code_fn);
    registry.register_eager("UNICODE",     code_fn::unicode_fn);
    registry.register_eager("EXACT",       exact::exact_fn);
    registry.register_eager("T",           t_fn::t_fn);
}
