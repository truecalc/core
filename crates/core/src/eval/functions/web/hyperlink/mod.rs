use crate::eval::functions::check_arity;
use crate::types::Value;

/// `HYPERLINK(url, [link_label])` — returns the link_label (or url if no label).
///
/// In a formula evaluator context (no browser rendering), HYPERLINK returns the
/// display text: the second argument if provided, otherwise the first argument.
pub fn hyperlink_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    if args.len() == 2 {
        args[1].clone()
    } else {
        args[0].clone()
    }
}

#[cfg(test)]
mod tests;
