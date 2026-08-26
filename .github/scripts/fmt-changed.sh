#!/usr/bin/env bash
# .github/scripts/fmt-changed.sh
#
# rustfmt ratchet: the set of rustfmt-clean files may only grow.
#
# This tree predates any formatting gate — `cargo fmt --all -- --check`
# disagrees with roughly two thirds of it — so a whole-workspace check would
# fail on files nobody touched, and no rustfmt.toml makes the tree clean
# (measured; the tree is unformatted, not formatted to a non-default profile).
#
# So the rule is per file, relative to the merge base:
#
#   added file          -> must be rustfmt-clean
#   file clean at base  -> must still be rustfmt-clean
#   file dirty at base  -> grandfathered, reported but not enforced
#
# That stops new drift without forcing a reformat of legacy files, which would
# mean rewriting code unrelated to the change (see CLAUDE.md, "Surgical
# Changes"). Grandfathered files clear as people choose to run --fix on them.
#
# Usage:
#   fmt-changed.sh [--fix] [base-ref]
#
#   --fix       rewrite every non-clean changed file in place, grandfathered
#               ones included
#   base-ref    ref to diff against (default: origin/main)
#
# Each file is fed to rustfmt on stdin so exactly that file is formatted.
# Passing a path instead makes rustfmt follow the file's `mod` declarations and
# reformat its children, dragging untouched files into the diff.

set -euo pipefail

FIX=0
if [[ "${1:-}" == "--fix" ]]; then
  FIX=1
  shift
fi
BASE="${1:-origin/main}"

cd "$(git rev-parse --show-toplevel)"

EDITION="$(grep -m1 '^edition' Cargo.toml | cut -d'"' -f2)"
echo "$(rustfmt --version) | edition ${EDITION} | base ${BASE}"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
CHANGED="$WORK/changed"
FAILED="$WORK/failed"
GRANDFATHERED="$WORK/grandfathered"
FIXED="$WORK/fixed"
: > "$FAILED"
: > "$GRANDFATHERED"
: > "$FIXED"

# Returns 0 when the file on stdin is already rustfmt-clean.
is_clean() {
  rustfmt --edition "$EDITION" --emit stdout < "$1" > "$WORK/fmt"
  cmp -s "$WORK/fmt" "$1"
}

git diff --name-only --diff-filter=ACMR "${BASE}...HEAD" -- '*.rs' > "$CHANGED"

if [[ ! -s "$CHANGED" ]]; then
  echo "No .rs files changed — nothing to check."
  exit 0
fi

echo "Checking $(wc -l < "$CHANGED" | tr -d ' ') changed .rs file(s)."

while IFS= read -r f; do
  [[ -f "$f" ]] || continue

  if is_clean "$f"; then
    continue
  fi

  if [[ $FIX -eq 1 ]]; then
    cp "$WORK/fmt" "$f"
    echo "$f" >> "$FIXED"
    continue
  fi

  # Not clean now. Was it clean at the merge base? A file that did not exist
  # there is new, and new code is held to the standard unconditionally.
  if git show "${BASE}:${f}" > "$WORK/base" 2>/dev/null && ! is_clean "$WORK/base"; then
    echo "$f" >> "$GRANDFATHERED"
  else
    echo "$f" >> "$FAILED"
  fi
done < "$CHANGED"

if [[ $FIX -eq 1 ]]; then
  if [[ -s "$FIXED" ]]; then
    echo "Formatted $(wc -l < "$FIXED" | tr -d ' ') file(s):"
    sed 's/^/  /' "$FIXED"
  else
    echo "All changed files were already rustfmt-clean."
  fi
  exit 0
fi

if [[ -s "$GRANDFATHERED" ]]; then
  echo
  echo "$(wc -l < "$GRANDFATHERED" | tr -d ' ') changed file(s) were already not"
  echo "rustfmt-clean before this branch — not enforced, listed for visibility:"
  sed 's/^/  /' "$GRANDFATHERED"
fi

if [[ ! -s "$FAILED" ]]; then
  echo
  echo "No new formatting drift."
  exit 0
fi

echo
echo "::error::$(wc -l < "$FAILED" | tr -d ' ') file(s) introduce formatting drift (added, or rustfmt-clean at ${BASE} and no longer clean)."
sed 's/^/  /' "$FAILED"
echo
echo "Fix with: bash .github/scripts/fmt-changed.sh --fix ${BASE}"
echo

while IFS= read -r f; do
  echo "--- $f"
  diff -u "$f" <(rustfmt --edition "$EDITION" --emit stdout < "$f") || true
done < "$FAILED"

exit 1
