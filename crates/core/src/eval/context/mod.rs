use std::collections::HashMap;
use crate::types::Value;

/// Holds the named variable bindings for a formula evaluation.
///
/// All keys are stored and looked up in uppercase, so variable names are
/// case-insensitive (`A1`, `a1`, and `A1` all refer to the same binding).
pub struct Context {
    pub vars: HashMap<String, Value>,
    /// Pinned "now" for volatile date functions (`NOW`, `TODAY`), as a
    /// local-time spreadsheet serial datetime (integer part = day serial,
    /// fraction = time of day). `None` ⇒ the functions read the ambient
    /// local clock. Set via [`crate::Engine::evaluate_at`] (P1.4, issue #526).
    pub now_serial: Option<f64>,
    /// Pinned "now" as an absolute UTC instant (nanoseconds since the Unix
    /// epoch), for the zone-aware `TZNOW`. `None` ⇒ `TZNOW` reads the ambient
    /// UTC clock. Set from the workbook's `timestamp_ms` during recalc; distinct
    /// from `now_serial` (a tz-naive local serial for `NOW`/`TODAY`).
    pub now_utc_nanos: Option<i64>,
    /// Per-cell RNG key for deterministic draws: (seed, sheet_index, row, col).
    /// Set by the workbook recalc loop; None for bare Engine::evaluate calls.
    pub rng_cell: Option<(u64, u32, u32, u32)>,
}

impl Context {
    /// Create a `Context` from a map of variable name → value.
    /// Keys are normalised to uppercase on insertion.
    pub fn new(vars: HashMap<String, Value>) -> Self {
        let normalized = vars.into_iter()
            .map(|(k, v)| (k.to_uppercase(), v))
            .collect();
        Self { vars: normalized, now_serial: None, now_utc_nanos: None, rng_cell: None }
    }

    /// Create an empty `Context` with no variable bindings.
    pub fn empty() -> Self {
        Self { vars: HashMap::new(), now_serial: None, now_utc_nanos: None, rng_cell: None }
    }

    /// Look up a variable by name (case-insensitive). Returns `Value::Empty` if not found.
    pub fn get(&self, name: &str) -> Value {
        self.vars
            .get(&name.to_uppercase())
            .cloned()
            .unwrap_or(Value::Empty)
    }

    /// Look up a binding by name (case-insensitive), distinguishing an absent
    /// binding (`None`) from one explicitly bound to [`Value::Empty`]
    /// (`Some(Value::Empty)`). Used by the evaluator to decide whether a
    /// reference should fall through to the resolver.
    pub fn lookup(&self, name: &str) -> Option<Value> {
        self.vars.get(&name.to_uppercase()).cloned()
    }

    /// Insert or overwrite a binding. Returns the previous value if one existed.
    pub fn set(&mut self, name: String, value: Value) -> Option<Value> {
        self.vars.insert(name.to_uppercase(), value)
    }

    /// Remove a binding. Used to restore context after lambda/let evaluation.
    pub fn remove(&mut self, name: &str) {
        self.vars.remove(&name.to_uppercase());
    }

    /// SplitMix64-based PRF: draw a value in [0,1) for the given draw index.
    /// Uses the per-cell key when available; falls back to SystemTime for
    /// bare Engine::evaluate calls (non-deterministic).
    pub fn draw_rand(&self, draw_index: u32) -> f64 {
        if let Some((seed, si, r, c)) = self.rng_cell {
            let key = self.splitmix_chain(seed, si, r, c, draw_index);
            // Use upper 32 bits for [0,1)
            (key >> 32) as f64 / (u32::MAX as f64 + 1.0)
        } else {
            // Non-deterministic fallback for bare Engine::evaluate.
            // Mix draw_index into the time seed so RANDARRAY cells differ.
            let s = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(12345);
            let key = Self::mix64(s ^ Self::mix64(draw_index as u64));
            (key >> 32) as f64 / (u32::MAX as f64 + 1.0)
        }
    }

    /// Draw `count` values in [0,1), each using its own draw_index.
    pub fn draw_rand_n(&self, count: usize) -> Vec<f64> {
        (0..count as u32).map(|i| self.draw_rand(i)).collect()
    }

    fn splitmix_chain(&self, seed: u64, si: u32, r: u32, c: u32, draw: u32) -> u64 {
        let mut h = seed;
        for part in [si as u64, r as u64, c as u64, draw as u64] {
            h = Self::mix64(h ^ Self::mix64(part));
        }
        h
    }

    fn mix64(mut x: u64) -> u64 {
        x = x.wrapping_add(0x9e3779b97f4a7c15);
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        x ^ (x >> 31)
    }
}

#[cfg(test)]
mod tests;
