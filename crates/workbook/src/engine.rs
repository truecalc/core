//! Engine flavor for workbooks.
//!
//! The flavor enum is owned by `truecalc-core` (the single source of truth);
//! the workbook layer re-exports it as [`EngineFlavor`] so there is exactly one
//! enum across the two crates and no drift is possible (issue #567). Core derives
//! the serde impls under its `serde` feature with `rename_all = "lowercase"`, so
//! the JSON wire strings stay `"sheets"` | `"excel"` as pinned by the workbook
//! JSON schema v1 (spec section 2).
//!
//! Engine flavor is explicit and required on every public entry point; there is
//! no default (ADR 2026-04-27-engine-flavor-explicit-everywhere). It is the
//! workbook value serialized as the required top-level `engine` field.

pub use truecalc_core::EngineFlavor;
