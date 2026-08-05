use chrono::Local;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::eval::functions::date::serial::date_to_serial;
use crate::parser::ast::Expr;
use crate::types::Value;

/// `TODAY()` — returns the current local date as a date-typed serial number.
///
/// Volatile. Registered lazy so it can read the evaluation context: when the
/// context carries a pinned `now_serial` (set via `Engine::evaluate_at`,
/// P1.4 issue #526), the date is `now_serial.floor()` — deterministic.
/// Without a pin it falls back to the ambient local clock.
pub fn today_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(e) = check_arity_len(args.len(), 0, 0) {
        return e;
    }
    let serial = match ctx.ctx.now_serial {
        Some(now) => now.floor(),
        None => date_to_serial(Local::now().date_naive()),
    };
    Value::Date(serial)
}

#[cfg(test)]
mod tests;
