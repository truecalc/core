//! Every `.rs` file under a crate's `src/` must be reachable from that crate's
//! root via `mod` declarations.
//!
//! A file nothing declares is invisible to the compiler, so `cargo test`,
//! `cargo clippy` and `cargo fmt` never visit it. That is how a complete
//! `INDEX` implementation and several test files sat in the tree for months
//! looking compiled and tested while being neither (issue #885). This test
//! walks the module tree from each crate root and fails on anything it cannot
//! reach, so the next orphan is caught when it is introduced.
//!
//! Note: `#[path = "..."]` module declarations are not resolved. The repo does
//! not use them; if one is added, the target will be reported here as an
//! orphan and this walker needs to learn about it.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    // crates/core -> crates -> <repo root>
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("truecalc-core lives at <root>/crates/core")
        .to_path_buf()
}

/// Every crate root in the workspace: `src/lib.rs`, `src/main.rs` and any
/// `src/bin/*.rs`, for each `crates/*` member plus `xtask`.
fn crate_roots(root: &Path) -> Vec<PathBuf> {
    let mut src_dirs: Vec<PathBuf> = Vec::new();
    let mut crate_dirs: Vec<PathBuf> = fs::read_dir(root.join("crates"))
        .expect("crates/ exists")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    crate_dirs.sort();
    crate_dirs.push(root.join("xtask"));
    for dir in crate_dirs {
        let src = dir.join("src");
        if src.is_dir() {
            src_dirs.push(src);
        }
    }

    let mut roots = Vec::new();
    for src in src_dirs {
        for name in ["lib.rs", "main.rs"] {
            let candidate = src.join(name);
            if candidate.is_file() {
                roots.push(candidate);
            }
        }
        let bin = src.join("bin");
        if bin.is_dir() {
            let mut bins: Vec<PathBuf> = fs::read_dir(&bin)
                .expect("readable bin dir")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "rs"))
                .collect();
            bins.sort();
            roots.append(&mut bins);
        }
    }
    roots
}

/// Module names declared by `mod name;` (as opposed to `mod name { .. }`).
/// Line-oriented on purpose: the declarations in this repo are one per line,
/// and the alternative is a Rust parser.
fn declared_modules(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with("//") || !line.ends_with(';') {
            continue;
        }
        let mut rest = line;
        for prefix in ["pub(crate)", "pub(super)", "pub(self)", "pub"] {
            if let Some(stripped) = rest.strip_prefix(prefix) {
                rest = stripped.trim_start();
                break;
            }
        }
        let Some(rest) = rest.strip_prefix("mod ") else {
            continue;
        };
        let name = rest.trim_end_matches(';').trim();
        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            names.push(name.to_string());
        }
    }
    names
}

fn visit(path: &Path, seen: &mut BTreeSet<PathBuf>) {
    if !path.is_file() || !seen.insert(path.to_path_buf()) {
        return;
    }
    let source = fs::read_to_string(path).expect("readable Rust source");
    let dir = path.parent().expect("file has a parent");
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    // Children of lib.rs/main.rs/mod.rs sit beside them; children of `foo.rs`
    // sit in `foo/`.
    let child_dir = match stem {
        "lib" | "main" | "mod" => dir.to_path_buf(),
        other => dir.join(other),
    };
    for name in declared_modules(&source) {
        visit(&child_dir.join(format!("{name}.rs")), seen);
        visit(&child_dir.join(&name).join("mod.rs"), seen);
    }
}

fn all_sources(dir: &Path, out: &mut BTreeSet<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            all_sources(&path, out);
        } else if path.extension().is_some_and(|x| x == "rs") {
            out.insert(path);
        }
    }
}

#[test]
fn every_source_file_is_reachable_from_a_crate_root() {
    let root = repo_root();

    let mut reachable = BTreeSet::new();
    for entry in crate_roots(&root) {
        visit(&entry, &mut reachable);
    }

    let mut present = BTreeSet::new();
    for dir in fs::read_dir(root.join("crates"))
        .expect("crates/ exists")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .chain(std::iter::once(root.join("xtask")))
    {
        let src = dir.join("src");
        if src.is_dir() {
            all_sources(&src, &mut present);
        }
    }

    let orphans: Vec<String> = present
        .difference(&reachable)
        .map(|p| {
            p.strip_prefix(&root)
                .unwrap_or(p)
                .display()
                .to_string()
                .replace('\\', "/")
        })
        .collect();

    assert!(
        orphans.is_empty(),
        "{} source file(s) under crates/*/src are never compiled — nothing \
         declares them, so cargo test/clippy/fmt never see them. Either add \
         the `mod` declaration that reaches them or delete them:\n  {}",
        orphans.len(),
        orphans.join("\n  ")
    );
}
