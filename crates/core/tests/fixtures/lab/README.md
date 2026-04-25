# Lab

This directory is a staging area for test cases discovered during development —
bugs caught in the wild, edge cases surfaced during investigations, or regression
probes written while fixing an issue.

## Intent

When a bug or unexpected behavior is found, a test case capturing it lands here
first. These cases stay in the lab until they have been submitted to the fixtures
pipeline, run in Google Sheets to receive a canonical expected value, and placed
in the appropriate `../google_sheets/*.tsv` file. At that point the lab entry is
removed.

The lab is never the final home for a test case — it is the first stop on the
way to becoming part of the official record.

## What belongs here

- Formulas that exposed a bug during development or code review
- Edge cases discovered while investigating a conformance failure
- Regression probes for fixed issues, pending canonicalization via the GS pipeline

## What does not belong here

- Modifications to `../google_sheets/*.tsv` — those files are canonical reference
  data produced by the fixtures pipeline and must never be edited by hand
- Expected values that have not been verified in Google Sheets

## CI behavior

- All `../google_sheets/*.tsv` tests **must pass** before a PR can merge.
- All property-based tests **must pass** before a PR can merge.
- Lab tests produce a visible report in CI output but **do not block merging**.
  A lab failure means a known case is still open; it is not a regression.

## Graduating a case

Once a lab case is ready, submit its formula through the fixtures pipeline.
The pipeline runs it in Google Sheets, records the canonical result, and places
it in the appropriate `../google_sheets/*.tsv` file. The lab entry is then removed.
