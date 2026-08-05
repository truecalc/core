//! Unicode **simple** case folding (schema spec §2), the single source of
//! truth for every case-insensitive comparison in this crate: sheet-name
//! uniqueness, named-range uniqueness, and case-insensitive sheet lookup
//! (plan item 3.1 sheet management).
//!
//! Pinned to Unicode *simple* `Case_Folding` (not `to_lowercase()` / full
//! folding, which disagree on some case pairs and would break cross-surface
//! determinism for names differing only in such characters).

use icu_casemap::CaseMapperBorrowed;

/// Applies simple case folding to `s`, character by character.
pub fn simple_fold(folder: &CaseMapperBorrowed<'static>, s: &str) -> String {
    s.chars().map(|c| folder.simple_fold(c)).collect()
}
