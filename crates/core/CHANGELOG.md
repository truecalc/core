# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [7.0.3](https://github.com/truecalc/core/compare/truecalc-core-v7.0.2...truecalc-core-v7.0.3) - 2026-08-04

### Fixed

- *(statistical)* count dates in COUNT, and route MAXA/MINA through zoned_extreme

### Other

- Merge pull request #793 from truecalc/fix/780-781-statistical-date-handling

## [7.0.2](https://github.com/truecalc/core/compare/truecalc-core-v7.0.1...truecalc-core-v7.0.2) - 2026-07-29

### Fixed

- *(google)* graduate the passing MIN-with-sparkline row out of bugs.tsv
- *(statistical)* captured MAX/MIN/MAXA/MINA blank, date and empty-array rows
- *(conformance)* duplicate the guard locally instead of super::
- *(statistical)* let dates participate in MAX/MIN/MAXA/MINA
- *(statistical)* MAX/MAXA/MINA answer 0 for an all-blank array

### Other

- *(conformance)* give the reporter the authored-cells guard the runners have
- *(statistical)* separate captured from extrapolated in the date rule
- *(statistical)* state the blank-only capture and where its rows live

## [7.0.1](https://github.com/truecalc/core/compare/truecalc-core-v7.0.0...truecalc-core-v7.0.1) - 2026-07-28

### Fixed

- *(statistical)* stop a date-only array answering 0 in MAX
- *(statistical)* withdraw the blank-array rule, keep the empty-array one
- *(statistical)* apply the absent-vs-numberless rule to MAX, MINA and MAXA
- *(statistical)* MIN returns #REF! for an empty array, matching MAX

## [7.0.0](https://github.com/truecalc/core/compare/truecalc-core-v6.1.0...truecalc-core-v7.0.0) - 2026-07-28

### Added

- *(google)* [**breaking**] SPARKLINE — parse and validate the in-cell chart ([#766](https://github.com/truecalc/core/pull/766))

### Fixed

- *(google)* SPARKLINE conformance rows ([#766](https://github.com/truecalc/core/pull/766))

### Other

- *(conformance)* enforce SPARKLINE's fixture coverage again
- Merge pull request #770 from truecalc/feat/766-sparkline-code
- *(conformance)* let SPARKLINE land before its fixture rows

## [6.1.0](https://github.com/truecalc/core/compare/truecalc-core-v6.0.1...truecalc-core-v6.1.0) - 2026-07-25

### Added

- *(core)* rewrite sheet-qualified formula refs on sheet rename

### Fixed

- correct case-fold doc claim, add LET-shadowing test
- fully flatten nested arrays in remaining statistical functions
- preserve column orientation for vertical ranges in elementwise ops

### Other

- Merge pull request #763 from truecalc/fix/724-vertical-range-spill

## [6.0.1](https://github.com/truecalc/core/compare/truecalc-core-v6.0.0...truecalc-core-v6.0.1) - 2026-07-22

### Other

- remove hardcoded function counts, add live badges + download badges

## [6.0.0](https://github.com/truecalc/core/compare/truecalc-core-v5.0.3...truecalc-core-v6.0.0) - 2026-07-22

### Added

- accept #REF! and the error-literal family as parser tokens

### Fixed

- *(parser)* make error_literal case-insensitive and avoid per-parse allocation

## [5.0.3](https://github.com/truecalc/core/compare/truecalc-core-v5.0.1...truecalc-core-v5.0.3) - 2026-07-21

### Fixed

- *(parser)* trim leading whitespace after '(' in parenthesised expressions

### Other

- release v5.0.2

## [5.0.2](https://github.com/truecalc/core/compare/truecalc-core-v5.0.1...truecalc-core-v5.0.2) - 2026-07-21

### Fixed

- *(parser)* trim leading whitespace after '(' in parenthesised expressions

## [5.0.1](https://github.com/truecalc/core/compare/truecalc-core-v5.0.0...truecalc-core-v5.0.1) - 2026-07-21

### Fixed

- *(parser)* start function-argument span at its first token

### Other

- Merge pull request #749 from truecalc/fix/746-arg-leading-space

## [5.0.0](https://github.com/truecalc/core/compare/truecalc-core-v4.0.0...truecalc-core-v5.0.0) - 2026-07-20

### Added

- *(core,workbook)* reach the per-node eval hook through workbook recalc
- *(eval)* fire lambda parameter-binding events from the HOF apply_lambda helper
- *(eval)* carry Span on the per-node evaluation hook
- *(eval)* opt-in per-node evaluation hook

### Fixed

- *(eval)* MAP double-eval, error propagation, and $-normalization in HOF lambda binding

## [4.0.0](https://github.com/truecalc/core/compare/truecalc-core-v3.3.0...truecalc-core-v4.0.0) - 2026-07-15

### Added

- *(core)* carry optional diagnostic messages on eval errors

### Fixed

- *(core)* propagate ErrorMsg everywhere + exact Sheets wording

## [3.3.0](https://github.com/truecalc/core/compare/truecalc-core-v3.2.0...truecalc-core-v3.3.0) - 2026-07-15

### Added

- *(date)* return NOW() as a date-typed value

### Other

- Merge pull request #727 from truecalc/feat/726-now-datetime-type

## [3.2.0](https://github.com/truecalc/core/compare/truecalc-core-v3.1.0...truecalc-core-v3.2.0) - 2026-07-15

### Fixed

- *(core)* SEQUENCE(N)/SEQUENCE(N,1) return an N×1 column vector

### Other

- Merge pull request #723 from truecalc/fix/707-column-vector-orientation

## [3.1.0](https://github.com/truecalc/core/compare/truecalc-core-v3.0.0...truecalc-core-v3.1.0) - 2026-07-15

### Added

- *(wasm-workbook)* expose translateFormula on the WebAssembly workbook binding ([#717](https://github.com/truecalc/core/pull/717))

## [3.0.0](https://github.com/truecalc/core/compare/truecalc-core-v2.0.1...truecalc-core-v3.0.0) - 2026-07-13

### Added

- *(core)* expose Engine::translate_formula (closes #709)
- *(core)* add translate_text splice entry point ([#709](https://github.com/truecalc/core/pull/709))
- *(core)* scope-aware ref collection skips LET/LAMBDA bindings ([#709](https://github.com/truecalc/core/pull/709))
- *(core)* add shift_ref_text with per-corner #REF! ([#709](https://github.com/truecalc/core/pull/709))
- *(core)* add shift_addr for translate_formula ([#709](https://github.com/truecalc/core/pull/709))
- add dollar_cell_ref tokenizer for $-anchored cell addresses

### Fixed

- *(core)* use sort_by_key instead of sort_by in translate_text ([#709](https://github.com/truecalc/core/pull/709))
- *(core)* correct translate_text test expectations to (d_row, d_col) order ([#709](https://github.com/truecalc/core/pull/709))
- *(core)* silence expected dead_code on shift_addr pending Task 2 ([#709](https://github.com/truecalc/core/pull/709))
- make evaluation-time lookup keys $-insensitive
- parse $-absolute references (closes #708)
- [**breaking**] add $ absolute/relative markers to CellAddr

## [2.0.1](https://github.com/truecalc/core/compare/truecalc-core-v2.0.0...truecalc-core-v2.0.1) - 2026-07-05

### Fixed

- add verified MODE.MULT/ARRAYFORMULA fixture rows now that #699/#700 are fixed
- ARRAYFORMULA broadcasts LEN/UPPER/ISNUMBER/IF, MODE.MULT returns all modes
- remove 7 stale MODE.MULT multi-mode rows from statistical.tsv
- verify and migrate remaining 32 stale bugs.tsv array/math rows
- remove 109 more stale bugs.tsv rows superseded by category TSVs
- remove 258 stale bugs.tsv rows superseded by array.tsv

### Other

- Merge pull request #697 from truecalc/fix/array-bugs-tsv-stale-rows

## [2.0.0](https://github.com/truecalc/core/compare/truecalc-core-v1.0.2...truecalc-core-v2.0.0) - 2026-06-23

### Added

- *(core)* MIN/MAX/SORT participation for zone-aware values
- *(core)* TZOVERLAP — working-hours overlap across N zones
- *(core)* flagship N-timezone compare + display
- *(core)* TZNOW (deterministic clock), TZINWINDOW, TZCANONICAL
- *(core)* timezone arithmetic — TZDIFF and TZADD
- *(core)* timezone construction, introspection and extraction functions
- *(core)* add foundational timezone functions
- *(core)* add Value::Zoned type and core-crate plumbing

### Fixed

- *(operator)* make Value::Date comparable as its serial number

### Other

- Merge pull request #686 from truecalc/feat/685-reframe-core-readme
- Merge pull request #684 from truecalc/feat/669-tz-overlap
- Merge pull request #672 from truecalc/feat/671-date-comparison-fix

## [1.0.2](https://github.com/truecalc/core/compare/truecalc-core-v1.0.0...truecalc-core-v1.0.2) - 2026-06-10

### Other

- release v1.0.1
- rustdoc, READMEs, cookbook, migration guide ([#548](https://github.com/truecalc/core/pull/548))

## [1.0.1](https://github.com/truecalc/core/compare/truecalc-core-v1.0.0...truecalc-core-v1.0.1) - 2026-06-10

### Other

- rustdoc, READMEs, cookbook, migration guide ([#548](https://github.com/truecalc/core/pull/548))

## [0.9.0](https://github.com/truecalc/core/compare/truecalc-core-v0.8.0...truecalc-core-v0.9.0) - 2026-06-10

### Added

- *(eval)* implement FREQUENCY (#592 theme)

### Fixed

- *(conformance)* flip info_conformance to blocking (256/256 pass)
- *(fixtures)* move SHEETS() harness-dependent rows from info.tsv to bugs.tsv
- *(clippy)* collapse Bool match arms in large/small (Rust 1.96 compat)
- *(conformance)* statistical.tsv passes — KURT/MODE.MULT/HYPGEOM.DIST + unit test updates
- *(varpa/tests/edge)* direct text -> #VALUE! not 0
- *(vara/tests/edge)* direct text -> #VALUE! not 0
- *(stdevpa/tests/edge)* direct text -> #VALUE! not 0
- *(stdeva/tests/edge)* direct text -> #VALUE! not 0
- *(stat_helpers)* AVERAGEA direct text empty->0, non-empty non-parseable->#VALUE!
- *(average)* direct-arg bool/text coercion, array skips bool/text
- *(distributions_impl)* clone ErrorKind in collect_weights_arg
- *(stat_helpers)* clone ErrorKind (not Copy)
- *(varpa)* use collect_nums_a_direct
- *(vara)* use collect_nums_a_direct
- *(stdevpa)* use collect_nums_a_direct
- *(stdeva)* use collect_nums_a_direct
- *(distributions)* AVERAGE.WEIGHTED scalar pairs, bool weights, negative weights
- *(distributions)* text arg coercion, df truncation, AVERAGE.WEIGHTED, BINOM.INV
- *(kurt)* use collect_nums_direct for GS coercion
- *(skew_p)* use collect_nums_direct for GS coercion
- *(skew)* use collect_nums_direct for GS coercion
- *(devsq)* use collect_nums_direct for GS coercion
- *(var_p)* use collect_nums_direct for GS coercion
- *(var_s)* use collect_nums_direct for GS coercion
- *(stdev_p)* use collect_nums_direct for GS coercion
- *(stdev_s)* use collect_nums_direct for GS coercion
- *(stdev_s)* use collect_nums_direct
- *(averagea)* use collect_nums_a_direct for GS-correct coercion
- *(avedev)* use collect_nums_direct for GS-correct bool/text coercion
- *(statistical)* add collect_nums_direct with GS direct-arg coercion semantics
- *(eval)* statistical conformance — align with 2026-06-08 fixtures
- *(fixtures)* remove 14 unfixable statistical rows; add STEYX to bugs.tsv
- *(clippy)* use is_empty() instead of len() >= 1 in row_col.rs
- *(array)* remove duplicate INDEX registration — INDEX moved to lookup registry
- *(index)* remove unused import; no behavioral change
- *(index)* coerce Bool args; handle row/col=0 (FALSE→0, TRUE→1)
- *(index)* re-register INDEX function in lookup registry
- *(formulatext)* propagate #REF! from argument instead of returning #N/A
- *(index,match)* INDEX negative row->VALUE!; MATCH handles column vectors
- *(lookup)* array form returns last-row value; full linear scan for unsorted ranges
- *(lookup)* value_compare handles Bool ordering for LOOKUP boolean search
- *(choose)* coerce bool TRUE→1, FALSE→#NUM!, empty→#NUM!
- *(eval)* lookup conformance — align with 2026-06-08 fixtures
- move lookup harness-dependent rows (INDIRECT cell content, OFFSET+INDIRECT, SHEET) to bugs.tsv
- remove harness-dependent rows from lookup.tsv
- *(conformance)* reporter handles date-type expected rows
- GS ignores places for negative BIN2OCT; fix unit test input to use positive
- restore j-suffix for IMCONJUGATE/IMEXP; fix format_oct places check for negatives
- always output 'i' suffix from IMLN; add text-float tolerance in conformance
- *(eval)* engineering conformance — align with 2026-06-08 fixtures
- *(eval)* clippy -- remove dead values_equal_1d, collapse nested if let
- *(eval)* filter conformance round 2 -- row 6 and row 8
- *(eval)* filter conformance -- align with 2026-06-08 fixtures
- *(clippy)* resolve 6 clippy warnings in text functions
- *(eval)* regexreplace GS semantics -- include zero-length matches after non-empty matches
- *(eval)* searchb prefix-match semantics, revert regexreplace, skip non-formula TSV rows
- *(test)* update unit tests to match fixture-driven behavior changes
- *(eval)* text conformance round 2 -- searchb tilde-escape, substitute occurrence=0, trim non-breaking space, TEXT format tokens, regexreplace empty-match
- *(eval)* ErrorKind does not impl Copy -- use clone() in regex fns
- *(eval)* text conformance -- align with 2026-06-08 fixtures
- *(lint)* use RangeInclusive::contains in yearfrac basis check
- *(eval)* prevent %m/%d/%Y matching 2-digit year inputs in DATEVALUE/serial parsing
- *(eval)* add Datelike trait import to fix compile errors in serial.rs and datevalue/mod.rs
- *(eval)* date conformance round 3 -- DATEVALUE month-name/2-digit-year, ISOWEEKNUM negative, YEARFRAC bad basis
- *(eval)* date conformance round 2 -- TIMEVALUE AM/PM, DATEVALUE month-name, ISOWEEKNUM negative, weekend mask validation
- *(eval)* date conformance -- align with 2026-06-08 fixtures
- *(lint)* collapse nested if in parse_complex to satisfy clippy
- *(eval)* IMPOWER preserve tiny residuals for rows 1039/1040/1053
- *(eval)* fix 7 remaining math conformance + 6 unit test regressions
- *(eval)* fix remaining 18 math conformance failures
- *(test)* update countblank tests for lazy fn signature
- *(eval)* math conformance — align with 2026-06-08 fixtures
- *(clippy)* elide needless lifetime, use is_empty, use range contains
- *(conformance)* revert info to report-only — 2 SHEETS()=19 rows are harness-dependent
- *(eval)* info conformance round 2 — ISREF/INDIRECT/registry fixes
- *(eval)* info conformance — align with 2026-06-08 fixtures
- *(criterion)* use is_ok_and to satisfy clippy
- *(criterion)* NumEq/NumNe coerce text cells to number
- *(eval)* AND/OR/XOR/SUM array-argument conformance
- *(eval)* logical conformance — align with 2026-06-08 fixtures
- *(parser)* add caret-exponent aliases for CONVERT area/volume units
- *(parser)* parser conformance — align with 2026-06-08 fixtures
- *(eval)* operator conformance — align with 2026-06-08 fixtures
- *(eval)* array coercion/error-kind batch (#592 theme 1)
- *(eval)* array error-kind alignment — ARRAY_CONSTRAIN/WRAPCOLS/WRAPROWS/TOCOL/TOROW (#592 theme 1)
- *(eval)* HOF lambda misuse returns #N/A not #VALUE! (theme 1)

### Other

- *(varpa)* text->0 per AVERAGEA semantics
- *(vara)* text->0 per AVERAGEA semantics
- *(stdevpa)* text->0 per AVERAGEA semantics
- *(stdeva)* text->0 per AVERAGEA semantics
- *(var_p)* update for direct-arg coercion semantics
- *(var_p)* update for direct-arg coercion semantics
- *(var_s)* update for direct-arg coercion semantics
- *(var_s)* update for direct-arg coercion semantics
- *(stdev_p)* update for direct-arg coercion semantics
- *(stdev_p)* update for direct-arg coercion semantics
- *(stdev_s)* update for direct-arg coercion semantics
- *(stdev_s)* update for direct-arg coercion semantics
- *(averagea)* update tests for direct-arg coercion semantics
- *(devsq)* update tests for direct-arg coercion semantics
- *(devsq)* update tests for direct-arg coercion semantics
- *(avedev)* update tests for direct-arg coercion semantics
- *(avedev)* update tests for direct-arg coercion semantics
- merge main into fix/conformance-financial
- Merge branch 'main' into fix/conformance-engineering
- *(conformance)* flip math to blocking — baseline CI run
- *(conformance)* flip database category to blocking
- *(array)* align unit/integration tests with fixture behavior
- Merge branch 'main' into feat/gs-snapshot-2026-06-08
- *(conformance)* gate refreshed-fixture categories report-only

## [0.8.0](https://github.com/truecalc/core/compare/truecalc-core-v0.7.0...truecalc-core-v0.8.0) - 2026-06-08

### Fixed

- *(workbook)* unify EngineFlavor with truecalc-core's flavor enum ([#567](https://github.com/truecalc/core/pull/567))

### Other

- *(workbook)* correct COUNT-skip comment to reference filed issue #584
- *(workbook)* seed Resolver from input-model sidecar (#575 / #532)

## [0.7.0](https://github.com/truecalc/core/compare/truecalc-core-v0.6.5...truecalc-core-v0.7.0) - 2026-06-08

### Added

- *(core)* Resolver trait, evaluate_with_resolver, extract_refs (P1.3)
- P1.2 reference grammar — Sheet1!A1, quoted sheets, cross-sheet ranges, named refs

### Fixed

- quote TRUE/FALSE sheet names in canonical display

### Other

- *(core)* fix EmptyResolver intra-doc link; clarify workbook.tsv report-only rationale
- Merge branch 'main' into feat/526-value-completeness
- Merge branch 'main' into feat/524-reference-grammar

## [0.6.5](https://github.com/truecalc/core/compare/truecalc-core-v0.6.4...truecalc-core-v0.6.5) - 2026-06-08

### Added

- engine-explicit API — Engine::sheets()/excel() required entry points

### Fixed

- regenerate workbook.tsv after GAS detectType fix (pipeline re-export)

### Other

- register workbook.tsv as report-only conformance category (P1.5 harness)
- P1.5 conformance fixtures — cross-sheet refs, named ranges, date types (pipeline-generated)

## [0.6.4](https://github.com/truecalc/core/compare/truecalc-core-v0.6.2...truecalc-core-v0.6.4) - 2026-04-26

### Fixed

- *(ci)* explain nextest vs formula-eval counts; remove oracle terminology

### Other

- release v0.6.3
- *(fixtures)* remove legacy m1-m4 xlsx fixtures and align lab with source-first layout

## [0.6.3](https://github.com/truecalc/core/compare/truecalc-core-v0.6.2...truecalc-core-v0.6.3) - 2026-04-25

### Fixed

- *(ci)* explain nextest vs formula-eval counts; remove oracle terminology

### Other

- *(fixtures)* remove legacy m1-m4 xlsx fixtures and align lab with source-first layout

## [0.6.2](https://github.com/truecalc/core/compare/truecalc-core-v0.6.0...truecalc-core-v0.6.2) - 2026-04-25

### Fixed

- *(clippy)* use RangeInclusive::contains for basis bounds check
- *(ci)* move platform-sensitive complex trig rows back to bugs.tsv
- *(financial)* EOM convention, clean price AI, CUMPRINC type normalization, date coercion
- *(financial)* XIRR Brent fallback for hard-to-converge rates
- *(financial)* DURATION/MDURATION negative coupon/yield, TBILLPRICE discount>=1, TBILLYIELD >1yr
- *(financial)* validation fixes for RECEIVED, DISC, YIELDDISC, INTRATE, YIELDMAT
- *(financial)* DDB factor<=0 and negative cost, DB period>life validation
- *(financial)* rewrite VDB with correct fractional periods and input validation
- *(financial)* rewrite AMORLINC with proper period 0, basis validation, and date proration
- *(financial)* skip booleans/text in MIRR and NPV array cash flows
- *(financial)* input validation for ACCRINT, PRICEDISC, PRICEMAT, CUMIPMT, CUMPRINC
- *(financial)* TBILLEQ compound formula for DSM > 182 days + discount bounds
- *(financial)* IRR ignores text strings in array literals (GS/Excel behaviour)
- *(financial)* robust convergence for IRR and RATE via Brent fallback
- *(financial)* remove .round() from DOLLARDE/DOLLARFR
- *(conformance)* reclaim 1033 passing rows from bugs.tsv

### Other

- release v0.6.0
- add lab staging area, CI separation guard, and drop "oracle" language
- *(conformance)* restore immutable google_sheets fixtures, introduce lab/

## [0.6.1](https://github.com/truecalc/core/compare/truecalc-core-v0.6.0...truecalc-core-v0.6.1) - 2026-04-25

### Fixed

- *(clippy)* use RangeInclusive::contains for basis bounds check
- *(ci)* move platform-sensitive complex trig rows back to bugs.tsv
- *(financial)* EOM convention, clean price AI, CUMPRINC type normalization, date coercion
- *(financial)* XIRR Brent fallback for hard-to-converge rates
- *(financial)* DURATION/MDURATION negative coupon/yield, TBILLPRICE discount>=1, TBILLYIELD >1yr
- *(financial)* validation fixes for RECEIVED, DISC, YIELDDISC, INTRATE, YIELDMAT
- *(financial)* DDB factor<=0 and negative cost, DB period>life validation
- *(financial)* rewrite VDB with correct fractional periods and input validation
- *(financial)* rewrite AMORLINC with proper period 0, basis validation, and date proration
- *(financial)* skip booleans/text in MIRR and NPV array cash flows
- *(financial)* input validation for ACCRINT, PRICEDISC, PRICEMAT, CUMIPMT, CUMPRINC
- *(financial)* TBILLEQ compound formula for DSM > 182 days + discount bounds
- *(financial)* IRR ignores text strings in array literals (GS/Excel behaviour)
- *(financial)* robust convergence for IRR and RATE via Brent fallback
- *(financial)* remove .round() from DOLLARDE/DOLLARFR
- *(conformance)* reclaim 1033 passing rows from bugs.tsv

### Other

- add lab staging area, CI separation guard, and drop "oracle" language
- *(conformance)* restore immutable google_sheets fixtures, introduce lab/

## [0.6.0](https://github.com/truecalc/core/compare/truecalc-core-v0.5.0...truecalc-core-v0.6.0) - 2026-04-25

### Added

- *(fixtures)* google_sheets snapshot 2026-04-23 (~11K test cases)

### Fixed

- *(conformance)* enable financial_conformance, remove google fixtures
- *(conformance)* move 6 complex-number precision rows to bugs.tsv
- *(conformance)* strip spurious header rows, fix 3 panics, move unimplemented rows to bugs.tsv, add google_conformance test
- remove CELL from standalone evaluator and dead stubs from filter/operator

### Other

- Merge pull request #506 from truecalc/feat/446-447-remove-cell-and-dead-stubs

## [0.5.0](https://github.com/truecalc/core/compare/truecalc-core-v0.4.19...truecalc-core-v0.5.0) - 2026-04-22

### Added

- *(core)* add Engine::google_sheets() conformance factory

### Fixed

- match GS behavior for inline arrays in SUMIFS, AVERAGEIFS, MAXIFS, MINIFS, SUBTOTAL, and CONCAT

### Other

- separate test files from production code

## [0.4.19](https://github.com/truecalc/core/compare/truecalc-core-v0.4.17...truecalc-core-v0.4.19) - 2026-04-21

### Fixed

- update integration tests to expect Error(Ref) for FREQUENCY and MODE.MULT
- resolve 7 conformance bugs (MARGINOFERROR, FREQUENCY, MODE.MULT, IMCOTH, IMTANH)
- support inline array args for PRODUCT, SUBTOTAL, SUMIFS, AVERAGEIFS, MAXIFS, MINIFS, DSUM, DAVERAGE
- unwrap array results to first element in scalar evaluation context

### Other

- Merge pull request #495 from truecalc/release-plz-2026-04-21T05-55-29Z
- re-evaluate all fixtures against GAS oracle; restore 29 oracle-verified coverage rows
- Merge pull request #497 from truecalc/feat/488-array-scalar-context

## [0.4.18](https://github.com/truecalc/core/compare/truecalc-core-v0.4.17...truecalc-core-v0.4.18) - 2026-04-21

### Fixed

- support inline array args for PRODUCT, SUBTOTAL, SUMIFS, AVERAGEIFS, MAXIFS, MINIFS, DSUM, DAVERAGE
- unwrap array results to first element in scalar evaluation context

### Other

- re-evaluate all fixtures against GAS oracle; restore 29 oracle-verified coverage rows
- Merge pull request #497 from truecalc/feat/488-array-scalar-context

## [0.4.17](https://github.com/truecalc/core/compare/truecalc-core-v0.4.16...truecalc-core-v0.4.17) - 2026-04-21

### Fixed

- implement ROMAN(number, style) and fix boolean coercion in array arithmetic
- remove stale SORTBY descending entry from bugs.tsv

### Other

- Merge pull request #494 from truecalc/feat/485-im-return-type
- Merge pull request #493 from truecalc/feat/489-statistical-math-bugs
- Merge pull request #492 from truecalc/feat/490-roman-boolean-coercion
- Merge pull request #481 from truecalc/release-plz-2026-04-21T01-03-24Z

## [0.4.16](https://github.com/truecalc/core/compare/truecalc-core-v0.4.14...truecalc-core-v0.4.16) - 2026-04-21

### Added

- enable per-function conformance coverage gate (T3.12)
- complete array+filter fixture generator (T3.4)
- oracle-evaluate all 15 fixture categories via GAS web app
- complete registry integrity — remove context-limited functions and add alias
- panic on duplicate function registration at startup

### Fixed

- update XMATCH unit tests to reflect correct mode=1/-1 behavior
- BUG-19 BETA.INV accepts 3 args with optional lo/hi defaults to 0/1
- BUG-16/BUG-17 TEXT time formats and VALUE comma/percent/dollar parsing
- BUG-12/BUG-13/BUG-14 wildcard matching in XLOOKUP, XMATCH, MATCH
- BUG-11 SORTN now sorts and returns top N elements
- BUG-09/BUG-10/BUG-04 SORT/SORTBY/UNIQUE 1D array handling
- re-evaluate fixtures with corrected oracle and restore CI green
- add missing TSV fixtures so all conformance tests pass on Track 1 branch

### Other

- release v0.4.15
- Merge pull request #478 from truecalc/feat/453-array-filter-generator
- resolve merge conflicts with main (Track 1 + generators merged)
- remove duplicate function registrations before assertion lands

## [0.4.15](https://github.com/truecalc/core/compare/truecalc-core-v0.4.14...truecalc-core-v0.4.15) - 2026-04-21

### Added

- enable per-function conformance coverage gate (T3.12)
- complete array+filter fixture generator (T3.4)
- oracle-evaluate all 15 fixture categories via GAS web app
- complete registry integrity — remove context-limited functions and add alias
- panic on duplicate function registration at startup

### Fixed

- update XMATCH unit tests to reflect correct mode=1/-1 behavior
- BUG-19 BETA.INV accepts 3 args with optional lo/hi defaults to 0/1
- BUG-16/BUG-17 TEXT time formats and VALUE comma/percent/dollar parsing
- BUG-12/BUG-13/BUG-14 wildcard matching in XLOOKUP, XMATCH, MATCH
- BUG-11 SORTN now sorts and returns top N elements
- BUG-09/BUG-10/BUG-04 SORT/SORTBY/UNIQUE 1D array handling
- re-evaluate fixtures with corrected oracle and restore CI green
- add missing TSV fixtures so all conformance tests pass on Track 1 branch

### Other

- Merge pull request #478 from truecalc/feat/453-array-filter-generator
- resolve merge conflicts with main (Track 1 + generators merged)
- remove duplicate function registrations before assertion lands

## [0.4.14](https://github.com/truecalc/core/compare/truecalc-core-v0.4.12...truecalc-core-v0.4.14) - 2026-04-20

### Other

- release v0.4.13
- Merge pull request #439 from truecalc/fix/435-property-web-improvements
- fix and expand property_web.rs after code review

## [0.4.13](https://github.com/truecalc/core/compare/truecalc-core-v0.4.12...truecalc-core-v0.4.13) - 2026-04-19

### Other

- Merge pull request #439 from truecalc/fix/435-property-web-improvements
- fix and expand property_web.rs after code review

## [0.4.12](https://github.com/truecalc/core/compare/truecalc-core-v0.4.11...truecalc-core-v0.4.12) - 2026-04-19

### Added

- register web functions and enable conformance tests

### Other

- add property-based tests for web functions

## [0.4.11](https://github.com/truecalc/core/compare/truecalc-core-v0.4.9...truecalc-core-v0.4.11) - 2026-04-19

### Other

- release v0.4.10
- add battle tests for PPMT, XIRR, IPMT, VLOOKUP approx, SUMPRODUCT

## [0.4.10](https://github.com/truecalc/core/compare/truecalc-core-v0.4.9...truecalc-core-v0.4.10) - 2026-04-19

### Other

- add battle tests for PPMT, XIRR, IPMT, VLOOKUP approx, SUMPRODUCT

## [0.4.8](https://github.com/truecalc/core/compare/truecalc-core-v0.4.6...truecalc-core-v0.4.8) - 2026-04-19

### Other

- Merge pull request #426 from truecalc/feat/coverage-array-operator-tests
- cover uncovered array functions and operator overflow paths
- add property tests for 7 missing categories
- *(operator)* add tests for comparison and unary operators

## [0.4.7](https://github.com/truecalc/core/compare/truecalc-core-v0.4.6...truecalc-core-v0.4.7) - 2026-04-19

### Other

- add property tests for 7 missing categories
- *(operator)* add tests for comparison and unary operators

## [0.4.6](https://github.com/truecalc/core/compare/truecalc-core-v0.4.5...truecalc-core-v0.4.6) - 2026-04-18

### Other

- Merge pull request #420 from truecalc/test/logical-core-coverage-415
- Merge pull request #419 from truecalc/test/financial-coverage-414
- Merge pull request #418 from truecalc/test/date-coverage-413
- Merge pull request #417 from truecalc/test/lookup-coverage-412
- *(lookup)* add edge test for mismatched lookup result range
- *(lookup)* add xlookup invalid match mode test
- *(lookup)* add tests for LOOKUP, XLOOKUP, XMATCH, ROW, COLUMN gaps

## [0.4.4](https://github.com/truecalc/core/compare/truecalc-core-v0.4.3...truecalc-core-v0.4.4) - 2026-04-18

### Added

- *(tests)* use CASES=500 constant for all proptest files — up from 256
- *(tests)* surface proptest case counts in CI output per property test
- *(conformance)* emit structured JSON summary — per-category pass/fail vs Google Sheets ([#378](https://github.com/truecalc/core/pull/378))
- *(proptest)* property tests for date, lookup, and array functions ([#376](https://github.com/truecalc/core/pull/376))

### Fixed

- *(tests)* replace all hardcoded 256 with CASES constant in eprintln strings

### Other

- *(proptest)* add array function property tests — SEQUENCE length and value invariants
- *(proptest)* add lookup function property tests — CHOOSE range invariants
- *(proptest)* add date function property tests — YEAR/MONTH/DAY roundtrip, DATEDIF invariants
- *(proptest)* add error propagation property tests for math and text functions
- *(proptest)* add idempotency, monotonicity, and round-trip properties for math functions
- *(proptest)* add idempotency and length properties for text functions

## [0.4.2](https://github.com/truecalc/core/compare/truecalc-core-v0.4.0...truecalc-core-v0.4.2) - 2026-04-17

### Fixed

- *(concat)* always return Value::Text, remove numeric oracle workaround

### Other

- release v0.4.1

## [0.4.1](https://github.com/truecalc/core/compare/truecalc-core-v0.4.0...truecalc-core-v0.4.1) - 2026-04-17

### Fixed

- *(concat)* always return Value::Text, remove numeric oracle workaround

## [0.4.0](https://github.com/truecalc/core/compare/truecalc-core-v0.3.12...truecalc-core-v0.4.0) - 2026-04-17

### Added

- implement M2/M3 lookup functions
- implement statistical distribution functions for M3 conformance

### Other

- Merge pull request #352 from truecalc/feat/334-m4-logical-lambda-impl
- Merge remote-tracking branch 'origin/main' into feat/334-m4-logical-lambda-impl
- Merge pull request #348 from truecalc/feat/325-m3-engineering-complex
- Merge pull request #347 from truecalc/feat/332-m4-filter
- resolve merge conflicts with origin/main in count/mod.rs and eval/mod.rs
- activate M2 text conformance
- resolve merge conflicts with main; fix SPLIT to return Value::Empty for empty parts
- re-trigger CI
- *(test)* convert split_fn/tests.rs to tests/success/failure/edge pattern
- add unit tests for SPLIT, TEXT, VALUE, COUNTA fixes
- activate M2 text conformance

## [0.3.12](https://github.com/truecalc/core/compare/truecalc-core-v0.3.10...truecalc-core-v0.3.12) - 2026-04-16

### Other

- Merge pull request #320 from truecalc/release-plz-2026-04-16T22-48-53Z
- activate M2 info and logical conformance tests
- Merge pull request #321 from truecalc/feat/120-activate-engineering-conformance
- activate M2 engineering conformance and fix 10 failures

## [0.3.11](https://github.com/truecalc/core/compare/truecalc-core-v0.3.10...truecalc-core-v0.3.11) - 2026-04-16

### Other

- Merge pull request #321 from truecalc/feat/120-activate-engineering-conformance
- activate M2 engineering conformance and fix 10 failures

## [0.3.10](https://github.com/truecalc/core/compare/truecalc-core-v0.3.9...truecalc-core-v0.3.10) - 2026-04-16

### Added

- implement BIN2DEC/HEX/OCT, DEC2BIN/HEX/OCT, HEX2BIN/DEC/OCT, OCT2BIN/DEC/HEX

## [0.3.9](https://github.com/truecalc/core/compare/truecalc-core-v0.3.8...truecalc-core-v0.3.9) - 2026-04-16

### Added

- implement BITAND, BITOR, BITXOR, BITLSHIFT, BITRSHIFT, DELTA, GESTEP

## [0.3.8](https://github.com/truecalc/core/compare/truecalc-core-v0.3.6...truecalc-core-v0.3.8) - 2026-04-16

### Added

- implement COUNTBLANK, COUNTUNIQUE, COUNTIFS, SUMIFS
- implement COMBIN, COMBINA, MULTINOMIAL, GCD, LCM
- implement SQRTPI, SUMSQ, FACTDOUBLE, SERIESSUM
- implement CEILING.MATH, CEILING.PRECISE, FLOOR.MATH, FLOOR.PRECISE, ISO.CEILING

### Fixed

- remove stray countifs/countunique/sumifs module stubs from mod.rs
- remove spurious pub mod entries from math mod.rs

### Other

- release v0.3.7
- Merge pull request #311 from truecalc/feat/105-math-base-conversion
- Merge pull request #310 from truecalc/feat/105-math-simple
- Merge pull request #309 from truecalc/feat/105-math-advanced-rounding

## [0.3.7](https://github.com/truecalc/core/compare/truecalc-core-v0.3.6...truecalc-core-v0.3.7) - 2026-04-16

### Added

- implement COUNTBLANK, COUNTUNIQUE, COUNTIFS, SUMIFS
- implement COMBIN, COMBINA, MULTINOMIAL, GCD, LCM
- implement SQRTPI, SUMSQ, FACTDOUBLE, SERIESSUM
- implement CEILING.MATH, CEILING.PRECISE, FLOOR.MATH, FLOOR.PRECISE, ISO.CEILING

### Fixed

- remove stray countifs/countunique/sumifs module stubs from mod.rs
- remove spurious pub mod entries from math mod.rs

### Other

- Merge pull request #311 from truecalc/feat/105-math-base-conversion
- Merge pull request #310 from truecalc/feat/105-math-simple
- Merge pull request #309 from truecalc/feat/105-math-advanced-rounding

## [0.3.6](https://github.com/truecalc/core/compare/truecalc-core-v0.3.4...truecalc-core-v0.3.6) - 2026-04-16

### Added

- implement LEFTB, RIGHTB, LENB, MIDB, FINDB, REPLACEB, SEARCHB
- implement REGEXMATCH, REGEXEXTRACT, REGEXREPLACE (#89 group C)
- implement ASC, JOIN, SPLIT, TEXTJOIN text functions
- implement LEFTB, RIGHTB, LENB, MIDB, FINDB, REPLACEB, SEARCHB
- implement ARABIC, ROMAN, CLEAN, FIXED, DOLLAR text functions

### Fixed

- replace regex with regex-lite to reduce WASM binary size
- remove stray module declarations from other PRs in text/mod.rs

### Other

- release v0.3.5
- replace flat tests.rs with tests/ subdirectory structure for arabic, clean, dollar, fixed, roman

## [0.3.5](https://github.com/truecalc/core/compare/truecalc-core-v0.3.4...truecalc-core-v0.3.5) - 2026-04-16

### Added

- implement LEFTB, RIGHTB, LENB, MIDB, FINDB, REPLACEB, SEARCHB
- implement REGEXMATCH, REGEXEXTRACT, REGEXREPLACE (#89 group C)
- implement ASC, JOIN, SPLIT, TEXTJOIN text functions
- implement LEFTB, RIGHTB, LENB, MIDB, FINDB, REPLACEB, SEARCHB
- implement ARABIC, ROMAN, CLEAN, FIXED, DOLLAR text functions

### Fixed

- replace regex with regex-lite to reduce WASM binary size
- remove stray module declarations from other PRs in text/mod.rs

### Other

- replace flat tests.rs with tests/ subdirectory structure for arabic, clean, dollar, fixed, roman

## [0.3.4](https://github.com/truecalc/core/compare/truecalc-core-v0.3.2...truecalc-core-v0.3.4) - 2026-04-16

### Added

- implement 14 order statistics functions (M2 #82)
- implement shape/distribution statistical functions (M2 #82)
- implement variance, stddev, covariance, and deviation statistical functions

### Fixed

- restore shape-stats module files deleted during rebase conflict resolution

### Other

- release v0.3.3
- enable m2 statistical conformance test (all 46 functions implemented)
- Merge pull request #298 from truecalc/feat/82-order-stats
- add unit tests for order statistics functions; remove shape-stats duplicates
- add edge tests and failure tests for shape-stats functions
- add unit tests for variance/stddev statistical functions

## [0.3.3](https://github.com/truecalc/core/compare/truecalc-core-v0.3.2...truecalc-core-v0.3.3) - 2026-04-16

### Added

- implement 14 order statistics functions (M2 #82)
- implement shape/distribution statistical functions (M2 #82)
- implement variance, stddev, covariance, and deviation statistical functions

### Fixed

- restore shape-stats module files deleted during rebase conflict resolution

### Other

- enable m2 statistical conformance test (all 46 functions implemented)
- Merge pull request #298 from truecalc/feat/82-order-stats
- add unit tests for order statistics functions; remove shape-stats duplicates
- add edge tests and failure tests for shape-stats functions
- add unit tests for variance/stddev statistical functions

## [0.3.2](https://github.com/truecalc/core/compare/truecalc-core-v0.3.0...truecalc-core-v0.3.2) - 2026-04-16

### Added

- implement CONVERT unit conversion function ([#176](https://github.com/truecalc/core/pull/176))
- implement TO_DATE, TO_DOLLARS, TO_PERCENT, TO_PURE_NUMBER, TO_TEXT parser functions

### Fixed

- truncate mi3 volume literal to suppress clippy::excessive_precision

### Other

- release v0.3.1
- Merge pull request #292 from truecalc/feat/176-convert
- activate m2_parser_conformance (all 6 parser functions implemented)

## [0.3.1](https://github.com/truecalc/core/compare/truecalc-core-v0.3.0...truecalc-core-v0.3.1) - 2026-04-16

### Added

- implement CONVERT unit conversion function ([#176](https://github.com/truecalc/core/pull/176))
- implement TO_DATE, TO_DOLLARS, TO_PERCENT, TO_PURE_NUMBER, TO_TEXT parser functions

### Fixed

- truncate mi3 volume literal to suppress clippy::excessive_precision

### Other

- Merge pull request #292 from truecalc/feat/176-convert
- activate m2_parser_conformance (all 6 parser functions implemented)

## [0.3.0](https://github.com/truecalc/core/compare/truecalc-core-v0.2.1...truecalc-core-v0.3.0) - 2026-04-16

### Added

- add Value::Date and implement ISDATE (closes #208)
- implement CELL function (closes #215)
- implement ISREF and ISFORMULA (closes #211, #213)
- *(math)* implement COUNTIF, SUMIF, AVERAGEIF (#273, #274, #275)
- *(text)* implement SEARCH with wildcard support ([#271](https://github.com/truecalc/core/pull/271))
- *(statistical)* implement COUNTBLANK ([#272](https://github.com/truecalc/core/pull/272))
- *(text)* implement PROPER function ([#270](https://github.com/truecalc/core/pull/270))
- *(parser)* add {} array literal syntax ([#269](https://github.com/truecalc/core/pull/269))
- *(date)* implement all 26 M2 date/time functions ([#75](https://github.com/truecalc/core/pull/75))
- *(date)* scaffold 26 date/time function stubs
- *(tests)* add Google Sheets oracle fixtures for M2, M3, M4 conformance

### Fixed

- align CELL info_type list with Google Sheets docs
- use range contains() for clippy manual_range_contains lint
- use is_empty() for clippy len_zero lint
- *(clippy)* remove unused ErrorKind import in countblank
- *(clippy)* resolve 4 clippy warnings in date functions
- *(tests)* mark M2/M3/M4 conformance tests as pending until implemented

### Other

- Expand M2 conformance coverage for issue #276 functions

## [0.2.1](https://github.com/truecalc/core/compare/truecalc-core-v0.2.0...truecalc-core-v0.2.1) - 2026-04-15

### Other

- Merge pull request #266 from truecalc/fix/registry-driven-list-functions
- replace static function tables with live registry reference

## [0.2.0](https://github.com/truecalc/core/compare/truecalc-core-v0.1.6...truecalc-core-v0.2.0) - 2026-04-15

### Fixed

- *(mcp)* make list_functions registry-driven, delete static catalogue

## [0.1.6](https://github.com/truecalc/core/compare/truecalc-core-v0.1.4...truecalc-core-v0.1.6) - 2026-04-15

### Other

- release v0.1.5
- Merge pull request #73 from truecalc/docs/readme-badges-and-usage
- add badges and per-crate READMEs for crates.io and npm

## [0.1.5](https://github.com/truecalc/core/compare/truecalc-core-v0.1.4...truecalc-core-v0.1.5) - 2026-04-15

### Other

- Merge pull request #73 from truecalc/docs/readme-badges-and-usage
- add badges and per-crate READMEs for crates.io and npm

## [0.1.4](https://github.com/truecalc/core/compare/truecalc-core-v0.1.3...truecalc-core-v0.1.4) - 2026-04-15

### Added

- *(eval)* implement wave-1 M1 functions (#49–#53, #56–#58)

### Fixed

- *(clippy)* use is_some() instead of if let Some(_) pattern
- *(conformance)* pass all 6 M1 oracle conformance test suites

## [0.1.3](https://github.com/truecalc/core/compare/truecalc-core-v0.1.1...truecalc-core-v0.1.3) - 2026-04-15

### Other

- release v0.1.2
- add M1 oracle conformance harness driven by Google Sheets

## [0.1.2](https://github.com/truecalc/core/compare/truecalc-core-v0.1.1...truecalc-core-v0.1.2) - 2026-04-15

### Other

- add M1 oracle conformance harness driven by Google Sheets

## [0.1.1](https://github.com/truecalc/core/compare/truecalc-core-v0.1.0...truecalc-core-v0.1.1) - 2026-04-15

### Fixed

- *(core)* evaluate() takes variables by reference ([#34](https://github.com/truecalc/core/pull/34))
