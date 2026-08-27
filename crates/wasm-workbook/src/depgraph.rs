//! Read-only dependency-graph queries for the JavaScript workbook surface.
//!
//! The workbook already derives a [`DependencyGraph`] on every recalculation
//! and drops it again — nothing is retained on the [`Workbook`]. This module
//! serializes that same derived view so a JavaScript host can answer the two
//! questions the grid itself cannot:
//!
//! - **What does this cell read?** — [`precedents_of`].
//! - **What breaks if I change this cell?** — [`dependents_of`].
//!
//! # Freshness
//!
//! [`Workbook`] caches no graph, so every query here calls
//! [`DependencyGraph::build`] against the workbook **as it is right now**. A
//! query therefore can never return a stale answer, and — because the graph is
//! a function of formula *text*, sheet names and named-range targets, not of
//! evaluated values — it is correct even before the first `recalc()` and
//! immediately after a `set()` / `clear()` / `defineName()` with no recalc in
//! between. The price is that a query costs `O(formula cells)` to build the
//! graph; it is not a cheap accessor, and a host that highlights precedents on
//! every selection change should expect that cost.
//!
//! # Bounds
//!
//! Every traversal is bounded on two axes — depth and emitted node count — and
//! **always reports whether it stopped early**. A caller may lower the bounds
//! but never raise them past [`MAX_MAX_DEPTH`] / [`MAX_MAX_NODES`]: clamping is
//! safe precisely because hitting the clamp sets `truncated`, so a truncated
//! answer can never be mistaken for a complete one.

use serde::Serialize;
use std::collections::HashSet;
use tsify_next::Tsify;

use truecalc_workbook::{Address, CellRef, DependencyGraph, NameTarget, Precedent, Workbook};

/// Depth used when a caller passes no `maxDepth`: direct precedents /
/// dependents only, the answer a formula bar or a "trace precedents" button
/// wants. Transitive traversal is opt-in.
pub const DEFAULT_MAX_DEPTH: u32 = 1;

/// Hard ceiling on `maxDepth`. A larger request is clamped to this and the
/// result reports `truncated` if the walk actually had further to go.
pub const MAX_MAX_DEPTH: u32 = 64;

/// Emitted-node budget used when a caller passes no `maxNodes`.
pub const DEFAULT_MAX_NODES: u32 = 1_000;

/// Hard ceiling on `maxNodes` — the guarantee that no dependency query can
/// return an unbounded payload, whatever the caller asks for.
pub const MAX_MAX_NODES: u32 = 10_000;

/// Why a traversal stopped early. Emitted as `truncatedBy` and `null` when the
/// result is complete.
const TRUNCATED_BY_MAX_NODES: &str = "maxNodes";
const TRUNCATED_BY_MAX_DEPTH: &str = "maxDepth";

/// A cell identified the way a JavaScript caller addresses one: the sheet name
/// **as written in the workbook** (original case, not the case-folded key the
/// graph indexes by) plus a plain A1 address.
#[derive(Tsify, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CellNode {
    /// Sheet name in the workbook's own casing — pass it straight back to
    /// `get`, `set` or another dependency query.
    pub sheet: String,
    /// Plain uppercase A1 address (`A1`, `BC42`) within `sheet`.
    pub a1: String,
}

/// What a named range currently points at, carried alongside the name so a
/// caller never has to resolve the indirection itself.
#[derive(Tsify, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NameTargetRef {
    /// The name targets a single cell.
    Cell { sheet: String, a1: String },
    /// The name targets a rectangular range.
    Range {
        sheet: String,
        /// Inclusive `A1:B2` range text, top-left corner first.
        range: String,
    },
    /// The name is not defined in this workbook, or its reference does not
    /// resolve. An explicit variant rather than an absent field: a missing
    /// target and an unresolvable one must not look the same.
    Unresolved,
}

/// One thing a formula reads, after sheet and name resolution.
///
/// A range is reported as a **single** range node, never expanded into its
/// cells: `SUM(A1:A100000)` is one precedent, which is what keeps the answer
/// bounded and is also how the engine itself stores the edge.
#[derive(Tsify, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PrecedentRef {
    /// A single-cell reference (`A1`, or `Sheet2!A1` — `sheet` is always
    /// populated, so a cross-sheet precedent is never flattened onto the
    /// queried cell's sheet).
    Cell { sheet: String, a1: String },
    /// A rectangular range reference (`A1:B2`, `Sheet2!A1:B2`).
    Range {
        sheet: String,
        /// Inclusive `A1:B2` range text, top-left corner first.
        range: String,
    },
    /// A workbook-scoped named range, with what it currently resolves to.
    Name {
        /// The name in the workbook's own casing — like `sheet` on `Cell` /
        /// `Range`, this is looked back up from the graph's case-folded key
        /// rather than echoed folded, so a UI never renders the wrong case.
        name: String,
        /// What the name currently points at. Always present;
        /// [`NameTargetRef::Unresolved`] when the name is undefined or its
        /// reference does not resolve.
        target: NameTargetRef,
    },
    /// A reference with no resolvable target — an unknown sheet or name, or a
    /// formula that failed to parse. It is reported rather than dropped
    /// because it is why the cell will evaluate to `#REF!` / `#NAME?`.
    Unresolved {
        /// The reference text as the graph recorded it.
        text: String,
    },
}

/// One precedent, with how far it sits from the queried cell.
#[derive(Tsify, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrecedentNode {
    /// `1` = read directly by the queried cell, `2` = read by something the
    /// queried cell reads, and so on. Always the **shortest** distance: a
    /// precedent reachable by several paths is reported once, at its nearest
    /// depth.
    pub depth: u32,
    /// What is read.
    pub reference: PrecedentRef,
}

/// One dependent, with how far it sits from the queried cell. Dependents are
/// always concrete formula cells — only a formula can read something.
#[derive(Tsify, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependentNode {
    /// `1` = reads the queried cell directly (as a cell, through a range that
    /// contains it, or through a named range whose target contains it), `2` =
    /// reads something that reads it, and so on. Always the shortest distance.
    pub depth: u32,
    /// Sheet name in the workbook's own casing.
    pub sheet: String,
    /// Plain uppercase A1 address within `sheet`.
    pub a1: String,
}

/// Answer to: what does this cell read?
#[derive(Tsify, Serialize, Debug, Clone, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PrecedentsResult {
    /// The cell that was queried, echoed back in canonical form.
    pub cell: CellNode,
    /// The precedents found, nearest depth first. **Always present**: a
    /// literal cell, an empty cell and a formula with no references all
    /// return an empty array — never a missing field — so absence of
    /// precedents is never confused with absence of an answer.
    pub precedents: Vec<PrecedentNode>,
    /// `true` when the walk stopped before exhausting the graph, i.e. the
    /// array above is a prefix of the real answer, not the whole of it.
    pub truncated: bool,
    /// Which bound stopped the walk. Present exactly when `truncated` is
    /// `true` — branch on `truncated`, which is always present; this is the
    /// detail, not the signal. If both bounds were reached this is
    /// `maxNodes`, the bound that actually ended the walk.
    #[tsify(optional, type = "\"maxNodes\" | \"maxDepth\"")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated_by: Option<String>,
}

/// Answer to: what breaks if I change this cell?
#[derive(Tsify, Serialize, Debug, Clone, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DependentsResult {
    /// The cell that was queried, echoed back in canonical form.
    pub cell: CellNode,
    /// The formula cells that would have to recalculate, nearest depth first.
    /// **Always present**, empty when nothing reads the cell.
    pub dependents: Vec<DependentNode>,
    /// `true` when the walk stopped before exhausting the graph.
    pub truncated: bool,
    /// Which bound stopped the walk. Present exactly when `truncated` is
    /// `true`.
    #[tsify(optional, type = "\"maxNodes\" | \"maxDepth\"")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated_by: Option<String>,
}

/// Resolves a query's `(sheet, a1)` arguments against `workbook`.
///
/// Errors on an unknown sheet or a malformed address rather than returning an
/// empty result: a typo'd sheet name that quietly answered `no precedents`
/// would be indistinguishable from the truth.
fn resolve_query_cell(
    workbook: &Workbook,
    sheet: &str,
    a1: &str,
) -> Result<(CellRef, CellNode), String> {
    let worksheet = workbook
        .sheet(sheet)
        .ok_or_else(|| format!("unknown sheet: {sheet:?}"))?;
    let addr = Address::from_a1(a1).ok_or_else(|| format!("invalid A1 address: {a1:?}"))?;
    let cell_node = CellNode {
        // Echo the workbook's own casing, not the caller's, so the answer is
        // addressable no matter how the sheet was spelled in the query.
        sheet: worksheet.name().to_string(),
        a1: addr.to_a1(),
    };
    Ok((CellRef::from_display_name(sheet, addr), cell_node))
}

/// The display (original-case) sheet name for a graph key, which carries the
/// case-folded form. Falls back to the folded name if the sheet has since been
/// removed — impossible here, since the graph is built from this same
/// workbook, but the fallback keeps the field non-empty by construction.
fn display_sheet(workbook: &Workbook, folded: &str) -> String {
    workbook
        .sheet(folded)
        .map(|s| s.name().to_string())
        .unwrap_or_else(|| folded.to_string())
}

fn cell_node(workbook: &Workbook, cell: &CellRef) -> CellNode {
    CellNode {
        sheet: display_sheet(workbook, &cell.sheet),
        a1: cell.addr.to_a1(),
    }
}

/// Clamps a requested depth into `1..=MAX_MAX_DEPTH`. The floor of 1 is
/// The display (original-case) name for a graph name key, which — like a
/// sheet — carries the case-folded form. Same fallback rationale as
/// [`display_sheet`]: impossible to miss here since the graph is built from
/// this same workbook, but keeps the field non-empty by construction.
fn display_name(workbook: &Workbook, folded: &str) -> String {
    workbook
        .name(folded)
        .map(|n| n.name.clone())
        .unwrap_or_else(|| folded.to_string())
}

/// deliberate: a depth of 0 would return an empty list with
/// `truncated: false`, which is the one answer shape that lies.
fn clamp_depth(requested: Option<u32>) -> u32 {
    requested
        .unwrap_or(DEFAULT_MAX_DEPTH)
        .clamp(1, MAX_MAX_DEPTH)
}

fn clamp_nodes(requested: Option<u32>) -> usize {
    requested.unwrap_or(DEFAULT_MAX_NODES).min(MAX_MAX_NODES) as usize
}

/// Converts a resolved [`Precedent`] into its wire form, resolving a name's
/// target through the graph so the caller never has to.
fn precedent_ref(workbook: &Workbook, graph: &DependencyGraph, prec: &Precedent) -> PrecedentRef {
    match prec {
        Precedent::Cell(c) => PrecedentRef::Cell {
            sheet: display_sheet(workbook, &c.sheet),
            a1: c.addr.to_a1(),
        },
        Precedent::Range(r) => PrecedentRef::Range {
            sheet: display_sheet(workbook, &r.sheet),
            range: format!("{}:{}", r.start.to_a1(), r.end.to_a1()),
        },
        Precedent::Name(name) => PrecedentRef::Name {
            name: display_name(workbook, name),
            target: match graph.name_target_of(name) {
                Some(NameTarget::Cell(c)) => NameTargetRef::Cell {
                    sheet: display_sheet(workbook, &c.sheet),
                    a1: c.addr.to_a1(),
                },
                Some(NameTarget::Range(r)) => NameTargetRef::Range {
                    sheet: display_sheet(workbook, &r.sheet),
                    range: format!("{}:{}", r.start.to_a1(), r.end.to_a1()),
                },
                None => NameTargetRef::Unresolved,
            },
        },
        Precedent::Unresolved(text) => PrecedentRef::Unresolved { text: text.clone() },
    }
}

/// What a cell at `(sheet, a1)` reads, walked up to `max_depth` levels and
/// `max_nodes` emitted precedents.
///
/// `max_depth`/`max_nodes` default to [`DEFAULT_MAX_DEPTH`] /
/// [`DEFAULT_MAX_NODES`] and are clamped to [`MAX_MAX_DEPTH`] /
/// [`MAX_MAX_NODES`]. Errors on an unknown sheet or malformed address.
pub fn precedents_of(
    workbook: &Workbook,
    sheet: &str,
    a1: &str,
    max_depth: Option<u32>,
    max_nodes: Option<u32>,
) -> Result<PrecedentsResult, String> {
    let (root, cell) = resolve_query_cell(workbook, sheet, a1)?;
    let max_depth = clamp_depth(max_depth);
    let max_nodes = clamp_nodes(max_nodes);
    let graph = DependencyGraph::build(workbook);

    let mut precedents: Vec<PrecedentNode> = Vec::new();
    // `Precedent` is `Hash + Eq`, so it is its own dedupe key: the same
    // reference reached by two paths is reported once, at its nearest depth.
    let mut emitted: HashSet<Precedent> = HashSet::new();
    // Cells whose precedent lists have already been expanded, so a cycle in the
    // graph terminates the walk instead of looping.
    let mut expanded: HashSet<CellRef> = HashSet::new();
    expanded.insert(root.clone());
    let mut frontier: Vec<CellRef> = vec![root];
    let mut truncated_by: Option<&'static str> = None;

    for depth in 1..=max_depth {
        if frontier.is_empty() {
            break;
        }
        let mut next: Vec<CellRef> = Vec::new();
        'frontier: for cell in &frontier {
            for prec in graph.precedents_of(cell).unwrap_or(&[]) {
                if !emitted.insert(prec.clone()) {
                    continue;
                }
                if precedents.len() >= max_nodes {
                    truncated_by = Some(TRUNCATED_BY_MAX_NODES);
                    break 'frontier;
                }
                precedents.push(PrecedentNode {
                    depth,
                    reference: precedent_ref(workbook, &graph, prec),
                });
                // Only formula cells have precedents of their own, so this is
                // exactly the set with anything left to say at depth + 1.
                for target in graph.formula_precedent_cells(prec) {
                    if expanded.insert(target.clone()) {
                        next.push(target);
                    }
                }
            }
        }
        if truncated_by.is_some() {
            break;
        }
        if depth == max_depth {
            // At the depth bound. Report truncation only if going one level
            // further would actually add something: a queued cell whose every
            // precedent is already reported adds nothing, and claiming
            // truncation there would cry wolf on the common `maxDepth: 1`
            // query. Probing one level costs a lookup per queued cell and
            // emits nothing, so the payload stays bounded either way.
            //
            // Sound as a termination test: a precedent's expansion targets are
            // queued when it is first reported, so a level that reports nothing
            // new also queues nothing new, and every deeper level is empty.
            let more = next.iter().any(|c| {
                graph
                    .precedents_of(c)
                    .unwrap_or(&[])
                    .iter()
                    .any(|p| !emitted.contains(p))
            });
            if more {
                truncated_by = Some(TRUNCATED_BY_MAX_DEPTH);
            }
            break;
        }
        frontier = next;
    }

    Ok(PrecedentsResult {
        cell,
        precedents,
        truncated: truncated_by.is_some(),
        truncated_by: truncated_by.map(str::to_string),
    })
}

/// What reads a cell at `(sheet, a1)` — the cells that must recalculate if it
/// changes — walked up to `max_depth` levels and `max_nodes` emitted cells.
///
/// Composes all three edge kinds: a direct cell reference, a range that
/// contains the cell, and a named range whose target contains it.
///
/// `max_depth`/`max_nodes` default to [`DEFAULT_MAX_DEPTH`] /
/// [`DEFAULT_MAX_NODES`] and are clamped to [`MAX_MAX_DEPTH`] /
/// [`MAX_MAX_NODES`]. Errors on an unknown sheet or malformed address.
pub fn dependents_of(
    workbook: &Workbook,
    sheet: &str,
    a1: &str,
    max_depth: Option<u32>,
    max_nodes: Option<u32>,
) -> Result<DependentsResult, String> {
    let (root, cell) = resolve_query_cell(workbook, sheet, a1)?;
    let max_depth = clamp_depth(max_depth);
    let max_nodes = clamp_nodes(max_nodes);
    let graph = DependencyGraph::build(workbook);

    let mut dependents: Vec<DependentNode> = Vec::new();
    let mut emitted: HashSet<CellRef> = HashSet::new();
    let mut expanded: HashSet<CellRef> = HashSet::new();
    expanded.insert(root.clone());
    let mut frontier: Vec<CellRef> = vec![root];
    let mut truncated_by: Option<&'static str> = None;

    for depth in 1..=max_depth {
        if frontier.is_empty() {
            break;
        }
        let mut next: Vec<CellRef> = Vec::new();
        'frontier: for cell in &frontier {
            for dep in graph.direct_dependents_of(cell) {
                if !emitted.insert(dep.clone()) {
                    continue;
                }
                if dependents.len() >= max_nodes {
                    truncated_by = Some(TRUNCATED_BY_MAX_NODES);
                    break 'frontier;
                }
                let node = cell_node(workbook, &dep);
                dependents.push(DependentNode {
                    depth,
                    sheet: node.sheet,
                    a1: node.a1,
                });
                if expanded.insert(dep.clone()) {
                    next.push(dep);
                }
            }
        }
        if truncated_by.is_some() {
            break;
        }
        if depth == max_depth {
            // Same one-level probe as `precedents_of`: only claim truncation
            // when a deeper walk would actually report another cell.
            let more = next.iter().any(|c| {
                graph
                    .direct_dependents_of(c)
                    .iter()
                    .any(|d| !emitted.contains(d))
            });
            if more {
                truncated_by = Some(TRUNCATED_BY_MAX_DEPTH);
            }
            break;
        }
        frontier = next;
    }

    Ok(DependentsResult {
        cell,
        dependents,
        truncated: truncated_by.is_some(),
        truncated_by: truncated_by.map(str::to_string),
    })
}
