//! Intent-based function discovery over the registry metadata.
//!
//! Given a natural-language query (e.g. `"monthly loan payment"`), rank the
//! user-facing functions by how well their metadata (name, category,
//! signature parameter names, and description) matches the query, so a caller
//! can find the right function without knowing the catalogue.
//!
//! The scoring is a deterministic, dependency-free keyword-overlap model: the
//! query and every metadata field are tokenized and light-stemmed, a small
//! curated synonym map bridges intent words to catalogue vocabulary, and each
//! query token contributes a fixed integer weight per field it hits. Identical
//! input always yields an identical ranking (integer scores, stable tie-break
//! on the function name).
//!
//! ```
//! use truecalc_core::{search_functions, Registry};
//!
//! let registry = Registry::new();
//! let matches = search_functions(&registry, "monthly loan payment", 5);
//! assert_eq!(matches.first().map(|m| m.name.as_str()), Some("PMT"));
//! ```

use crate::eval::functions::Registry;

/// A single ranked search result. All fields are owned so the result outlives
/// the borrow of the [`Registry`] it was produced from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionMatch {
    /// Canonical (upper-case) function name, e.g. `"PMT"`.
    pub name: String,
    /// Category the function belongs to, e.g. `"financial"`.
    pub category: String,
    /// Call signature, e.g. `"PMT(rate, nper, pv)"`.
    pub signature: String,
    /// One-line human description.
    pub description: String,
    /// Relevance score (higher is more relevant). Deterministic integer.
    pub score: u32,
}

// ── Scoring weights ─────────────────────────────────────────────────────────
// Integer weights keep ranking fully deterministic (no float ordering).

/// Query token equals the function name.
const W_NAME_EXACT: u32 = 100;
/// Function name begins with the query token (e.g. `sum` → `SUMIF`).
const W_NAME_PREFIX: u32 = 30;
/// Query token equals a description word.
const W_DESC_EXACT: u32 = 12;
/// A description word begins with the query token.
const W_DESC_PREFIX: u32 = 5;
/// Query token equals the category.
const W_CATEGORY: u32 = 8;
/// Query token equals a signature parameter word.
const W_SIGNATURE: u32 = 4;

/// Full weight applied to tokens the user actually typed (percent).
const WEIGHT_PRIMARY: u32 = 100;
/// Reduced weight applied to synonym-expanded tokens (percent).
const WEIGHT_SYNONYM: u32 = 60;

/// Rank the user-facing functions in `registry` against `query`.
///
/// Returns matches with a non-zero score, most relevant first. Ties are broken
/// by function name (ascending) so the ordering is stable and deterministic.
/// `limit` caps the number of results; `0` means "no cap" (return all matches).
pub fn search_functions(registry: &Registry, query: &str, limit: usize) -> Vec<FunctionMatch> {
    let query_tokens = expand_query(&tokenize(query));
    if query_tokens.is_empty() {
        return Vec::new();
    }

    let mut matches: Vec<FunctionMatch> = registry
        .list_functions()
        .filter_map(|(name, meta)| {
            let score = score_function(&query_tokens, name, meta.category, meta.signature, meta.description);
            (score > 0).then(|| FunctionMatch {
                name: name.to_string(),
                category: meta.category.to_string(),
                signature: meta.signature.to_string(),
                description: meta.description.to_string(),
                score,
            })
        })
        .collect();

    // Descending score, then ascending name for a stable, deterministic order.
    matches.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.name.cmp(&b.name)));

    if limit > 0 && matches.len() > limit {
        matches.truncate(limit);
    }
    matches
}

/// Score one function's metadata against the (already expanded) query tokens.
fn score_function(
    query_tokens: &[(String, u32)],
    name: &str,
    category: &str,
    signature: &str,
    description: &str,
) -> u32 {
    let name_stem = stem(&name.to_lowercase());
    let category_stem = stem(&category.to_lowercase());
    let desc_tokens = tokenize(description);
    let sig_tokens = tokenize(signature);

    let mut total: u32 = 0;
    for (qtoken, weight) in query_tokens {
        let mut field_score: u32 = 0;

        // Name (highest signal).
        if *qtoken == name_stem {
            field_score += W_NAME_EXACT;
        } else if qtoken.len() >= 3 && name_stem.starts_with(qtoken.as_str()) {
            field_score += W_NAME_PREFIX;
        }

        // Description — count each query token at most once (best hit).
        if desc_tokens.iter().any(|t| t == qtoken) {
            field_score += W_DESC_EXACT;
        } else if qtoken.len() >= 4 && desc_tokens.iter().any(|t| t.starts_with(qtoken.as_str())) {
            field_score += W_DESC_PREFIX;
        }

        // Category.
        if *qtoken == category_stem {
            field_score += W_CATEGORY;
        }

        // Signature parameter names.
        if sig_tokens.iter().any(|t| t == qtoken) {
            field_score += W_SIGNATURE;
        }

        total += field_score * weight / 100;
    }
    total
}

/// Split text into lower-cased, light-stemmed, stop-word-filtered tokens.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| stem(&t.to_lowercase()))
        .filter(|t| !is_stopword(t))
        .collect()
}

/// Very light stemmer: fold a trailing plural `s` (e.g. `periods` → `period`)
/// so singular/plural forms match symmetrically on both query and metadata.
fn stem(token: &str) -> String {
    if token.len() > 3 && token.ends_with('s') && !token.ends_with("ss") {
        token[..token.len() - 1].to_string()
    } else {
        token.to_string()
    }
}

/// Common English/filler words carry no discovery signal.
fn is_stopword(token: &str) -> bool {
    matches!(
        token,
        "a" | "an"
            | "the"
            | "of"
            | "for"
            | "to"
            | "in"
            | "on"
            | "and"
            | "or"
            | "my"
            | "me"
            | "i"
            | "want"
            | "need"
            | "get"
            | "find"
            | "how"
            | "do"
            | "that"
            | "this"
            | "with"
            | "by"
            | "is"
            | "it"
            | "given"
    )
}

/// Expand tokenized query terms with curated synonyms. Original terms keep the
/// primary weight; synonym-derived terms are added at a reduced weight. The
/// result is de-duplicated, keeping the highest weight per token.
fn expand_query(tokens: &[String]) -> Vec<(String, u32)> {
    let mut out: Vec<(String, u32)> = Vec::new();
    let mut push = |token: String, weight: u32| {
        if let Some(existing) = out.iter_mut().find(|(t, _)| *t == token) {
            if weight > existing.1 {
                existing.1 = weight;
            }
        } else {
            out.push((token, weight));
        }
    };

    for token in tokens {
        push(token.clone(), WEIGHT_PRIMARY);
        for syn in synonyms(token) {
            push(stem(syn), WEIGHT_SYNONYM);
        }
    }
    out
}

/// Curated intent → catalogue-vocabulary synonyms. Keys are already stemmed
/// (trailing plural `s` folded). Intentionally small and deterministic — this
/// is a discovery aid, not a thesaurus.
fn synonyms(stemmed_token: &str) -> &'static [&'static str] {
    match stemmed_token {
        "loan" | "mortgage" => &["payment", "pmt", "principal", "interest"],
        "payment" | "installment" => &["pmt"],
        "monthly" | "annual" | "yearly" | "periodic" => &["period", "rate"],
        "average" | "avg" => &["mean"],
        "mean" => &["average"],
        "total" | "add" => &["sum"],
        "tally" => &["count", "counta"],
        "biggest" | "largest" | "maximum" | "highest" => &["max"],
        "smallest" | "minimum" | "lowest" => &["min"],
        "lookup" | "search" => &["vlookup", "hlookup", "xlookup", "match"],
        "concatenate" | "join" | "combine" | "merge" => &["concat", "textjoin"],
        "uppercase" => &["upper"],
        "lowercase" => &["lower"],
        "current" => &["today", "now", "date"],
        "remainder" | "modulo" => &["mod"],
        "absolute" => &["abs"],
        "squareroot" | "sqrt" => &["sqrt"],
        "percentage" | "percent" => &["percentrank", "percentile"],
        "deviation" | "spread" => &["stdev", "var"],
        "correlation" => &["correl"],
        "future" => &["fv"],
        "present" => &["pv"],
        "depreciation" => &["sln", "ddb", "syd"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests;
